use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use itertools::Itertools;
use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::timed;
use plonky2::util::timing::TimingTree;
use rand::Rng;

use crate::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use crate::cross_table_lookup::Column;
use crate::lookup::{eval_lookups, eval_lookups_circuit, permuted_cols};
use crate::memory::columns::{
    is_channel, value_limb, ADDR_CONTEXT, ADDR_SEGMENT, ADDR_VIRTUAL, COLUMNS_TO_PAD,
    CONTEXT_FIRST_CHANGE, COUNTER, COUNTER_PERMUTED, IS_READ, NUM_COLUMNS, RANGE_CHECK,
    RANGE_CHECK_PERMUTED, SEGMENT_FIRST_CHANGE, TIMESTAMP, VIRTUAL_FIRST_CHANGE,
};
use crate::memory::NUM_CHANNELS;
use crate::permutation::PermutationPair;
use crate::stark::Stark;
use crate::util::trace_rows_to_poly_values;
use crate::vars::{StarkEvaluationTargets, StarkEvaluationVars};

pub(crate) const NUM_PUBLIC_INPUTS: usize = 0;

pub fn ctl_data<F: Field>() -> Vec<Column<F>> {
    let mut res =
        Column::singles([IS_READ, ADDR_CONTEXT, ADDR_SEGMENT, ADDR_VIRTUAL]).collect_vec();
    res.extend(Column::singles((0..8).map(value_limb)));
    res.push(Column::single(TIMESTAMP));
    res
}

pub fn ctl_filter<F: Field>(channel: usize) -> Column<F> {
    Column::single(is_channel(channel))
}

#[derive(Copy, Clone, Default)]
pub struct MemoryStark<F, const D: usize> {
    pub(crate) f: PhantomData<F>,
}

#[derive(Debug)]
pub struct MemoryOp<F> {
    pub channel_index: usize,
    pub timestamp: usize,
    pub is_read: bool,
    pub context: usize,
    pub segment: usize,
    pub virt: usize,
    pub value: [F; 8],
}

pub fn generate_random_memory_ops<F: RichField, R: Rng>(
    num_ops: usize,
    rng: &mut R,
) -> Vec<MemoryOp<F>> {
    let mut memory_ops = Vec::new();

    let mut current_memory_values: HashMap<(usize, usize, usize), [F; 8]> = HashMap::new();
    let num_cycles = num_ops / 2;
    for clock in 0..num_cycles {
        let mut used_indices = HashSet::new();
        let mut new_writes_this_cycle = HashMap::new();
        let mut has_read = false;
        for _ in 0..2 {
            let mut channel_index = rng.gen_range(0..NUM_CHANNELS);
            while used_indices.contains(&channel_index) {
                channel_index = rng.gen_range(0..NUM_CHANNELS);
            }
            used_indices.insert(channel_index);

            let is_read = if clock == 0 {
                false
            } else {
                !has_read && rng.gen()
            };
            has_read = has_read || is_read;

            let (context, segment, virt, vals) = if is_read {
                let written: Vec<_> = current_memory_values.keys().collect();
                let &(context, segment, virt) = written[rng.gen_range(0..written.len())];
                let &vals = current_memory_values
                    .get(&(context, segment, virt))
                    .unwrap();

                (context, segment, virt, vals)
            } else {
                // TODO: with taller memory table or more padding (to enable range-checking bigger diffs),
                // test larger address values.
                let mut context = rng.gen_range(0..40);
                let mut segment = rng.gen_range(0..8);
                let mut virt = rng.gen_range(0..20);
                while new_writes_this_cycle.contains_key(&(context, segment, virt)) {
                    context = rng.gen_range(0..40);
                    segment = rng.gen_range(0..8);
                    virt = rng.gen_range(0..20);
                }

                let val: [u32; 8] = rng.gen();
                let vals: [F; 8] = val.map(F::from_canonical_u32);

                new_writes_this_cycle.insert((context, segment, virt), vals);

                (context, segment, virt, vals)
            };

            let timestamp = clock * NUM_CHANNELS + channel_index;
            memory_ops.push(MemoryOp {
                channel_index,
                timestamp,
                is_read,
                context,
                segment,
                virt,
                value: vals,
            });
        }
        for (k, v) in new_writes_this_cycle {
            current_memory_values.insert(k, v);
        }
    }

    memory_ops
}

pub fn generate_first_change_flags<F: RichField>(
    context: &[F],
    segment: &[F],
    virtuals: &[F],
) -> (Vec<F>, Vec<F>, Vec<F>) {
    let num_ops = context.len();
    let mut context_first_change = Vec::with_capacity(num_ops);
    let mut segment_first_change = Vec::with_capacity(num_ops);
    let mut virtual_first_change = Vec::with_capacity(num_ops);
    for idx in 0..num_ops - 1 {
        let this_context_first_change = context[idx] != context[idx + 1];
        let this_segment_first_change =
            segment[idx] != segment[idx + 1] && !this_context_first_change;
        let this_virtual_first_change = virtuals[idx] != virtuals[idx + 1]
            && !this_segment_first_change
            && !this_context_first_change;

        context_first_change.push(F::from_bool(this_context_first_change));
        segment_first_change.push(F::from_bool(this_segment_first_change));
        virtual_first_change.push(F::from_bool(this_virtual_first_change));
    }

    context_first_change.push(F::ZERO);
    segment_first_change.push(F::ZERO);
    virtual_first_change.push(F::ZERO);

    (
        context_first_change,
        segment_first_change,
        virtual_first_change,
    )
}

pub fn generate_range_check_value<F: RichField>(
    context: &[F],
    segment: &[F],
    virtuals: &[F],
    timestamp: &[F],
    context_first_change: &[F],
    segment_first_change: &[F],
    virtual_first_change: &[F],
) -> (Vec<F>, usize) {
    let num_ops = context.len();
    let mut range_check = Vec::new();

    for idx in 0..num_ops - 1 {
        let this_address_unchanged = F::ONE
            - context_first_change[idx]
            - segment_first_change[idx]
            - virtual_first_change[idx];
        range_check.push(
            context_first_change[idx] * (context[idx + 1] - context[idx] - F::ONE)
                + segment_first_change[idx] * (segment[idx + 1] - segment[idx] - F::ONE)
                + virtual_first_change[idx] * (virtuals[idx + 1] - virtuals[idx] - F::ONE)
                + this_address_unchanged * (timestamp[idx + 1] - timestamp[idx] - F::ONE),
        );
    }
    range_check.push(F::ZERO);

    let max_diff = range_check.iter().map(F::to_canonical_u64).max().unwrap() as usize;

    (range_check, max_diff)
}

impl<F: RichField + Extendable<D>, const D: usize> MemoryStark<F, D> {
    pub(crate) fn generate_trace_rows(
        &self,
        mut memory_ops: Vec<MemoryOp<F>>,
    ) -> Vec<[F; NUM_COLUMNS]> {
        memory_ops.sort_by_key(|op| {
            (
                op.context.to_canonical_u64(),
                op.segment.to_canonical_u64(),
                op.virt.to_canonical_u64(),
                op.timestamp.to_canonical_u64(),
            )
        });

        let num_ops = memory_ops.len();

        let mut trace_cols = [(); NUM_COLUMNS].map(|_| vec![F::ZERO; num_ops]);
        for i in 0..num_ops {
            let MemoryOp {
                channel_index,
                timestamp,
                is_read,
                context,
                segment,
                virt,
                value,
            } = memory_ops[i];
            trace_cols[is_channel(channel_index)][i] = F::ONE;
            trace_cols[TIMESTAMP][i] = F::from_canonical_usize(timestamp);
            trace_cols[IS_READ][i] = F::from_bool(is_read);
            trace_cols[ADDR_CONTEXT][i] = F::from_canonical_usize(context);
            trace_cols[ADDR_SEGMENT][i] = F::from_canonical_usize(segment);
            trace_cols[ADDR_VIRTUAL][i] = F::from_canonical_usize(virt);
            for j in 0..8 {
                trace_cols[value_limb(j)][i] = value[j];
            }
        }

        self.generate_memory(&mut trace_cols);

        // The number of rows may have changed, if the range check required padding.
        let num_ops = trace_cols[0].len();

        let mut trace_rows = vec![[F::ZERO; NUM_COLUMNS]; num_ops];
        for (i, col) in trace_cols.iter().enumerate() {
            for (j, &val) in col.iter().enumerate() {
                trace_rows[j][i] = val;
            }
        }
        trace_rows
    }

    fn generate_memory(&self, trace_cols: &mut [Vec<F>]) {
        let num_trace_rows = trace_cols[0].len();

        let timestamp = &trace_cols[TIMESTAMP];
        let context = &trace_cols[ADDR_CONTEXT];
        let segment = &trace_cols[ADDR_SEGMENT];
        let virtuals = &trace_cols[ADDR_VIRTUAL];

        let (context_first_change, segment_first_change, virtual_first_change) =
            generate_first_change_flags(context, segment, virtuals);

        let (range_check_value, max_diff) = generate_range_check_value(
            context,
            segment,
            virtuals,
            timestamp,
            &context_first_change,
            &segment_first_change,
            &virtual_first_change,
        );
        let to_pad_to = (max_diff + 1).max(num_trace_rows).next_power_of_two();
        let to_pad = to_pad_to - num_trace_rows;

        trace_cols[CONTEXT_FIRST_CHANGE] = context_first_change;
        trace_cols[SEGMENT_FIRST_CHANGE] = segment_first_change;
        trace_cols[VIRTUAL_FIRST_CHANGE] = virtual_first_change;

        trace_cols[RANGE_CHECK] = range_check_value;

        for col in COLUMNS_TO_PAD {
            trace_cols[col].splice(0..0, vec![F::ZERO; to_pad]);
        }

        trace_cols[COUNTER] = (0..to_pad_to).map(|i| F::from_canonical_usize(i)).collect();

        let (permuted_inputs, permuted_table) =
            permuted_cols(&trace_cols[RANGE_CHECK], &trace_cols[COUNTER]);
        trace_cols[RANGE_CHECK_PERMUTED] = permuted_inputs;
        trace_cols[COUNTER_PERMUTED] = permuted_table;
    }

    pub fn generate_trace(&self, memory_ops: Vec<MemoryOp<F>>) -> Vec<PolynomialValues<F>> {
        let mut timing = TimingTree::new("generate trace", log::Level::Debug);

        // Generate the witness.
        let trace_rows = timed!(
            &mut timing,
            "generate trace rows",
            self.generate_trace_rows(memory_ops)
        );

        let trace_polys = timed!(
            &mut timing,
            "convert to PolynomialValues",
            trace_rows_to_poly_values(trace_rows)
        );

        timing.print();
        trace_polys
    }
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for MemoryStark<F, D> {
    const COLUMNS: usize = NUM_COLUMNS;
    const PUBLIC_INPUTS: usize = NUM_PUBLIC_INPUTS;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        vars: StarkEvaluationVars<FE, P, { Self::COLUMNS }, { Self::PUBLIC_INPUTS }>,
        yield_constr: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let one = P::from(FE::ONE);

        let timestamp = vars.local_values[TIMESTAMP];
        let addr_context = vars.local_values[ADDR_CONTEXT];
        let addr_segment = vars.local_values[ADDR_SEGMENT];
        let addr_virtual = vars.local_values[ADDR_VIRTUAL];
        let values: Vec<_> = (0..8).map(|i| vars.local_values[value_limb(i)]).collect();

        let next_timestamp = vars.next_values[TIMESTAMP];
        let next_is_read = vars.next_values[IS_READ];
        let next_addr_context = vars.next_values[ADDR_CONTEXT];
        let next_addr_segment = vars.next_values[ADDR_SEGMENT];
        let next_addr_virtual = vars.next_values[ADDR_VIRTUAL];
        let next_values: Vec<_> = (0..8).map(|i| vars.next_values[value_limb(i)]).collect();

        // Indicator that this is a real row, not a row of padding.
        // TODO: enforce that all padding is at the beginning.
        let valid_row: P = (0..NUM_CHANNELS)
            .map(|c| vars.local_values[is_channel(c)])
            .sum();

        let context_first_change = vars.local_values[CONTEXT_FIRST_CHANGE];
        let segment_first_change = vars.local_values[SEGMENT_FIRST_CHANGE];
        let virtual_first_change = vars.local_values[VIRTUAL_FIRST_CHANGE];
        let address_unchanged =
            one - context_first_change - segment_first_change - virtual_first_change;

        let range_check = vars.local_values[RANGE_CHECK];

        let not_context_first_change = one - context_first_change;
        let not_segment_first_change = one - segment_first_change;
        let not_virtual_first_change = one - virtual_first_change;
        let not_address_unchanged = one - address_unchanged;

        // First set of ordering constraint: first_change flags are boolean.
        yield_constr.constraint(context_first_change * not_context_first_change);
        yield_constr.constraint(segment_first_change * not_segment_first_change);
        yield_constr.constraint(virtual_first_change * not_virtual_first_change);
        yield_constr.constraint(address_unchanged * not_address_unchanged);

        // Second set of ordering constraints: no change before the column corresponding to the nonzero first_change flag.
        yield_constr
            .constraint_transition(segment_first_change * (next_addr_context - addr_context));
        yield_constr
            .constraint_transition(virtual_first_change * (next_addr_context - addr_context));
        yield_constr
            .constraint_transition(virtual_first_change * (next_addr_segment - addr_segment));
        yield_constr.constraint_transition(
            valid_row * address_unchanged * (next_addr_context - addr_context),
        );
        yield_constr.constraint_transition(
            valid_row * address_unchanged * (next_addr_segment - addr_segment),
        );
        yield_constr.constraint_transition(
            valid_row * address_unchanged * (next_addr_virtual - addr_virtual),
        );

        // Third set of ordering constraints: range-check difference in the column that should be increasing.
        let computed_range_check = context_first_change * (next_addr_context - addr_context - one)
            + segment_first_change * (next_addr_segment - addr_segment - one)
            + virtual_first_change * (next_addr_virtual - addr_virtual - one)
            + valid_row * address_unchanged * (next_timestamp - timestamp - one);
        yield_constr.constraint_transition(range_check - computed_range_check);

        // Enumerate purportedly-ordered log.
        for i in 0..8 {
            yield_constr
                .constraint(next_is_read * address_unchanged * (next_values[i] - values[i]));
        }

        eval_lookups(vars, yield_constr, RANGE_CHECK_PERMUTED, COUNTER_PERMUTED)
    }

    fn eval_ext_circuit(
        &self,
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
        vars: StarkEvaluationTargets<D, { Self::COLUMNS }, { Self::PUBLIC_INPUTS }>,
        yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let one = builder.one_extension();

        let addr_context = vars.local_values[ADDR_CONTEXT];
        let addr_segment = vars.local_values[ADDR_SEGMENT];
        let addr_virtual = vars.local_values[ADDR_VIRTUAL];
        let values: Vec<_> = (0..8).map(|i| vars.local_values[value_limb(i)]).collect();
        let timestamp = vars.local_values[TIMESTAMP];

        let next_addr_context = vars.next_values[ADDR_CONTEXT];
        let next_addr_segment = vars.next_values[ADDR_SEGMENT];
        let next_addr_virtual = vars.next_values[ADDR_VIRTUAL];
        let next_values: Vec<_> = (0..8).map(|i| vars.next_values[value_limb(i)]).collect();
        let next_is_read = vars.next_values[IS_READ];
        let next_timestamp = vars.next_values[TIMESTAMP];

        // Indicator that this is a real row, not a row of padding.
        let mut valid_row = vars.local_values[is_channel(0)];
        for c in 1..NUM_CHANNELS {
            valid_row = builder.add_extension(valid_row, vars.local_values[is_channel(c)]);
        }

        let context_first_change = vars.local_values[CONTEXT_FIRST_CHANGE];
        let segment_first_change = vars.local_values[SEGMENT_FIRST_CHANGE];
        let virtual_first_change = vars.local_values[VIRTUAL_FIRST_CHANGE];
        let address_unchanged = {
            let mut cur = builder.sub_extension(one, context_first_change);
            cur = builder.sub_extension(cur, segment_first_change);
            builder.sub_extension(cur, virtual_first_change)
        };

        let range_check = vars.local_values[RANGE_CHECK];

        let not_context_first_change = builder.sub_extension(one, context_first_change);
        let not_segment_first_change = builder.sub_extension(one, segment_first_change);
        let not_virtual_first_change = builder.sub_extension(one, virtual_first_change);
        let not_address_unchanged = builder.sub_extension(one, address_unchanged);
        let addr_context_diff = builder.sub_extension(next_addr_context, addr_context);
        let addr_segment_diff = builder.sub_extension(next_addr_segment, addr_segment);
        let addr_virtual_diff = builder.sub_extension(next_addr_virtual, addr_virtual);

        // First set of ordering constraint: traces are boolean.
        let context_first_change_bool =
            builder.mul_extension(context_first_change, not_context_first_change);
        yield_constr.constraint(builder, context_first_change_bool);
        let segment_first_change_bool =
            builder.mul_extension(segment_first_change, not_segment_first_change);
        yield_constr.constraint(builder, segment_first_change_bool);
        let virtual_first_change_bool =
            builder.mul_extension(virtual_first_change, not_virtual_first_change);
        yield_constr.constraint(builder, virtual_first_change_bool);
        let address_unchanged_bool =
            builder.mul_extension(address_unchanged, not_address_unchanged);
        yield_constr.constraint(builder, address_unchanged_bool);

        // Second set of ordering constraints: no change before the column corresponding to the nonzero first_change flag.
        let segment_first_change_check =
            builder.mul_extension(segment_first_change, addr_context_diff);
        yield_constr.constraint_transition(builder, segment_first_change_check);
        let virtual_first_change_check_1 =
            builder.mul_extension(virtual_first_change, addr_context_diff);
        yield_constr.constraint_transition(builder, virtual_first_change_check_1);
        let virtual_first_change_check_2 =
            builder.mul_extension(virtual_first_change, addr_segment_diff);
        yield_constr.constraint_transition(builder, virtual_first_change_check_2);
        let address_unchanged_check_1 = builder.mul_extension(address_unchanged, addr_context_diff);
        let address_unchanged_check_1_valid =
            builder.mul_extension(valid_row, address_unchanged_check_1);
        yield_constr.constraint_transition(builder, address_unchanged_check_1_valid);
        let address_unchanged_check_2 = builder.mul_extension(address_unchanged, addr_segment_diff);
        let address_unchanged_check_2_valid =
            builder.mul_extension(valid_row, address_unchanged_check_2);
        yield_constr.constraint_transition(builder, address_unchanged_check_2_valid);
        let address_unchanged_check_3 = builder.mul_extension(address_unchanged, addr_virtual_diff);
        let address_unchanged_check_3_valid =
            builder.mul_extension(valid_row, address_unchanged_check_3);
        yield_constr.constraint_transition(builder, address_unchanged_check_3_valid);

        // Third set of ordering constraints: range-check difference in the column that should be increasing.
        let context_diff = {
            let diff = builder.sub_extension(next_addr_context, addr_context);
            builder.sub_extension(diff, one)
        };
        let context_range_check = builder.mul_extension(context_first_change, context_diff);
        let segment_diff = {
            let diff = builder.sub_extension(next_addr_segment, addr_segment);
            builder.sub_extension(diff, one)
        };
        let segment_range_check = builder.mul_extension(segment_first_change, segment_diff);
        let virtual_diff = {
            let diff = builder.sub_extension(next_addr_virtual, addr_virtual);
            builder.sub_extension(diff, one)
        };
        let virtual_range_check = builder.mul_extension(virtual_first_change, virtual_diff);
        let timestamp_diff = {
            let diff = builder.sub_extension(next_timestamp, timestamp);
            builder.sub_extension(diff, one)
        };
        let timestamp_range_check = builder.mul_extension(address_unchanged, timestamp_diff);
        let timestamp_range_check = builder.mul_extension(valid_row, timestamp_range_check);

        let computed_range_check = {
            let mut sum = builder.add_extension(context_range_check, segment_range_check);
            sum = builder.add_extension(sum, virtual_range_check);
            builder.add_extension(sum, timestamp_range_check)
        };
        let range_check_diff = builder.sub_extension(range_check, computed_range_check);
        yield_constr.constraint_transition(builder, range_check_diff);

        // Enumerate purportedly-ordered log.
        for i in 0..8 {
            let value_diff = builder.sub_extension(next_values[i], values[i]);
            let zero_if_read = builder.mul_extension(address_unchanged, value_diff);
            let read_constraint = builder.mul_extension(next_is_read, zero_if_read);
            yield_constr.constraint(builder, read_constraint);
        }

        eval_lookups_circuit(
            builder,
            vars,
            yield_constr,
            RANGE_CHECK_PERMUTED,
            COUNTER_PERMUTED,
        )
    }

    fn constraint_degree(&self) -> usize {
        3
    }

    fn permutation_pairs(&self) -> Vec<PermutationPair> {
        vec![
            PermutationPair::singletons(RANGE_CHECK, RANGE_CHECK_PERMUTED),
            PermutationPair::singletons(COUNTER, COUNTER_PERMUTED),
        ]
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

    use crate::memory::memory_stark::MemoryStark;
    use crate::stark_testing::{test_stark_circuit_constraints, test_stark_low_degree};

    #[test]
    fn test_stark_degree() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        type S = MemoryStark<F, D>;

        let stark = S {
            f: Default::default(),
        };
        test_stark_low_degree(stark)
    }

    #[test]
    fn test_stark_circuit() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        type S = MemoryStark<F, D>;

        let stark = S {
            f: Default::default(),
        };
        test_stark_circuit_constraints::<F, C, S, D>(stark)
    }
}
