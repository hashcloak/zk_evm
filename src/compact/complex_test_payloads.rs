use eth_trie_utils::partial_trie::PartialTrie;

use super::compact_prestate_processing::{
    process_compact_prestate, process_compact_prestate_debug, CompactParsingResult,
    ProcessedCompactOutput,
};
use crate::{trace_protocol::TrieCompact, types::TrieRootHash};

pub(crate) const TEST_PAYLOAD_1: TestProtocolInputAndRoot = TestProtocolInputAndRoot { byte_str: "01055821033601462093b5945d1676df093446790fd31b20e7b12a2e8e5e09d068109616b0084a021e19e0c9bab240000005582103468288056310c82aa4c01a7e12a10f8111a0560e72b700555479031b86c357d0084101031a697e814758281972fcd13bc9707dbcd2f195986b05463d7b78426508445a0405582103b70e80538acdabd6137353b0f9d8d149f4dba91e8be2e7946e409bfdbe685b900841010558210389802d6ed1a28b049e9d4fe5334c5902fd9bc00c42821c82f82ee2da10be90800841010558200256274a27dd7524955417c11ecd917251cc7c4c8310f4c7e4bd3c304d3d9a79084a021e19e0c9bab2400000055820023ab0970b73895b8c9959bae685c3a19f45eb5ad89d42b52a340ec4ac204d190841010219102005582103876da518a393dbd067dc72abfa08d475ed6447fca96d92ec3f9e7eba503ca6100841010558210352688a8f926c816ca1e079067caba944f158e764817b83fc43594370ca9cf62008410105582103690b239ba3aaf993e443ae14aeffc44cf8d9931a79baed9fa141d0e4506e131008410102196573", root_str: "6a0673c691edfa4c4528323986bb43c579316f436ff6f8b4ac70854bbd95340b" };

pub(crate) const TEST_PAYLOAD_2: TestProtocolInputAndRoot = TestProtocolInputAndRoot { byte_str: "01055821033601462093b5945d1676df093446790fd31b20e7b12a2e8e5e09d068109616b0084a021e19e0c9bab240000005582103468288056310c82aa4c01a7e12a10f8111a0560e72b700555479031b86c357d0084101031a697e814758281972fcd13bc9707dbcd2f195986b05463d7b78426508445a0405582103b70e80538acdabd6137353b0f9d8d149f4dba91e8be2e7946e409bfdbe685b900841010558210389802d6ed1a28b049e9d4fe5334c5902fd9bc00c42821c82f82ee2da10be90800841010558200256274a27dd7524955417c11ecd917251cc7c4c8310f4c7e4bd3c304d3d9a790c014a021e0c000250c782fa00055820023ab0970b73895b8c9959bae685c3a19f45eb5ad89d42b52a340ec4ac204d1908410102191020055820021eec2b84f0ba344fd4b4d2f022469febe7a772c4789acfc119eb558ab1da3d08480de0b6b3a76400000558200276da518a393dbd067dc72abfa08d475ed6447fca96d92ec3f9e7eba503ca61084101021901200558210352688a8f926c816ca1e079067caba944f158e764817b83fc43594370ca9cf62008410105582103690b239ba3aaf993e443ae14aeffc44cf8d9931a79baed9fa141d0e4506e131008410102196573", root_str: "e779761e7f0cf4bb2b5e5a2ebac65406d3a7516d46798040803488825a01c19c" };

pub(crate) const TEST_PAYLOAD_3: TestProtocolInputAndRoot = TestProtocolInputAndRoot { byte_str: "01055821033601462093b5945d1676df093446790fd31b20e7b12a2e8e5e09d068109616b0084a021e19e0c9bab240000005582103468288056310c82aa4c01a7e12a10f8111a0560e72b700555479031b86c357d0084101031a697e814758281972fcd13bc9707dbcd2f195986b05463d7b78426508445a0405582103b70e80538acdabd6137353b0f9d8d149f4dba91e8be2e7946e409bfdbe685b900841010558210389802d6ed1a28b049e9d4fe5334c5902fd9bc00c42821c82f82ee2da10be90800841010558200256274a27dd7524955417c11ecd917251cc7c4c8310f4c7e4bd3c304d3d9a790c024a021e0a9cae36fa8e4788055820023ab0970b73895b8c9959bae685c3a19f45eb5ad89d42b52a340ec4ac204d1908410102191020055820021eec2b84f0ba344fd4b4d2f022469febe7a772c4789acfc119eb558ab1da3d08480f43fc2c04ee00000558200276da518a393dbd067dc72abfa08d475ed6447fca96d92ec3f9e7eba503ca61084101021901200558210352688a8f926c816ca1e079067caba944f158e764817b83fc43594370ca9cf62008410105582103690b239ba3aaf993e443ae14aeffc44cf8d9931a79baed9fa141d0e4506e131008410102196573", root_str: "6978d65a3f2fc887408cc28dbb796836ff991af73c21ea74d03a11f6cdeb119c" };

pub(crate) const TEST_PAYLOAD_4: TestProtocolInputAndRoot = TestProtocolInputAndRoot { byte_str: "01055821033601462093b5945d1676df093446790fd31b20e7b12a2e8e5e09d068109616b0084a021e19e0c9bab240000005582103468288056310c82aa4c01a7e12a10f8111a0560e72b700555479031b86c357d00841010359458c01cf05df7b300bb6768f77e774f47e91b1d1dd358c98b2f2118466f37305582103b70e80538acdabd6137353b0f9d8d149f4dba91e8be2e7946e409bfdbe685b900841010558210389802d6ed1a28b049e9d4fe5334c5902fd9bc00c42821c82f82ee2da10be90800841010558200256274a27dd7524955417c11ecd917251cc7c4c8310f4c7e4bd3c304d3d9a790c014a0218ae73977cea178000055820023ab0970b73895b8c9959bae685c3a19f45eb5ad89d42b52a340ec4ac204d1908410102191020055820021eec2b84f0ba344fd4b4d2f022469febe7a772c4789acfc119eb558ab1da3d0c0e49056b5974e248d87b700558200276da518a393dbd067dc72abfa08d475ed6447fca96d92ec3f9e7eba503ca6108410102190120035deadf02dd8344275283fee394945c5e15787054e0eef21f50c960fd913232970605582103f417f50fc699ebb817e23468e114836fb4578b6281ced73df8cbbfefb42724300701191c86037eea3a48563e7b938852aafc93d760d31a84ad520adf1128af576cdd65ee9a8e0605582103558c2c1ac06ad29eab5b631a2a76f7997030f5468deb7f384eb6e276208d04600701192b420558210352688a8f926c816ca1e079067caba944f158e764817b83fc43594370ca9cf62008410105582103690b239ba3aaf993e443ae14aeffc44cf8d9931a79baed9fa141d0e4506e131008410104592ef4608060405234801561001057600080fd5b50600436106104545760003560e01c806380947f8011610241578063bf529ca11161013b578063dd9bef60116100c3578063f279ca8111610087578063f279ca8114611161578063f4d1fc6114611191578063f58fc36a146111c1578063f6b0bbf7146111f1578063fde7721c1461122157610454565b8063dd9bef6014611071578063de97a363146110a1578063e9f9b3f2146110d1578063ea5141e614611101578063edf003cf1461113157610454565b8063ce3cf4ef1161010a578063ce3cf4ef14610f81578063d117320b14610fb1578063d51e7b5b14610fe1578063d53ff3fd14611011578063d93cd5581461104157610454565b8063bf529ca114610ec1578063c360aba614610ef1578063c420eb6114610f21578063c4bd65d514610f5157610454565b8063a18683cb116101c9578063b374012b1161018d578063b374012b14610dd1578063b3d847f214610e01578063b7b8620714610e31578063b81c148414610e61578063bdc875fc14610e9157610454565b8063a18683cb14610cf3578063a271b72114610d23578063a60a108714610d41578063a645c9c214610d71578063acaebdf614610da157610454565b8063962e4dc211610210578063962e4dc214610c0357806398456f3e14610c335780639a2b7c8114610c635780639cce7cf914610c93578063a040aec614610cc357610454565b806380947f8014610b43578063880eff3914610b73578063918a5fcd14610ba357806391e7b27714610bd357610454565b80633430ec061161035257806360e13cde116102da5780636f099c8d1161029e5780636f099c8d14610a5357806371d91d2814610a835780637b6e0b0e14610ab35780637c191d2014610ae35780637de8c6f814610b1357610454565b806360e13cde14610975578063613d0a82146109a557806363138d4f146109d5578063659bbb4f14610a055780636e7f1fe714610a2357610454565b806340fe26621161032157806340fe26621461088557806344cf3bc7146108b55780634a61af1f146108e55780634d2c74b3146109155780635590c2d91461094557610454565b80633430ec06146107d7578063371303c0146108075780633a411f12146108255780633a425dfc1461085557610454565b806318093b46116103e0578063219cddeb116103a4578063219cddeb146106e75780632294fc7f146107175780632871ef85146107475780632b21ef44146107775780632d34e798146107a757610454565b806318093b46146105f757806319b621d6146106275780631aba07ea146106575780631de2f343146106875780632007332e146106b757610454565b80630ba8a73b116104275780630ba8a73b146105195780631287a68c14610549578063135d52f7146105675780631581cf191461059757806316582150146105c757610454565b8063034aef7114610459578063050082f814610489578063087b4e84146104b95780630b3b996a146104e9575b600080fd5b610473600480360381019061046e9190612611565b611251565b604051610480919061264d565b60405180910390f35b6104a3600480360381019061049e9190612611565b61128c565b6040516104b0919061264d565b60405180910390f35b6104d360048036038101906104ce9190612611565b6112c7565b6040516104e0919061264d565b60405180910390f35b61050360048036038101906104fe91906127ae565b611301565b6040516105109190612876565b60405180910390f35b610533600480360381019061052e9190612611565b611328565b604051610540919061264d565b60405180910390f35b610551611364565b60405161055e919061264d565b60405180910390f35b610581600480360381019061057c9190612611565b61136d565b60405161058e919061264d565b60405180910390f35b6105b160048036038101906105ac9190612611565b6113a9565b6040516105be919061264d565b60405180910390f35b6105e160048036038101906105dc9190612611565b6113e4565b6040516105ee919061264d565b60405180910390f35b610611600480360381019061060c9190612611565b61143f565b60405161061e919061264d565b60405180910390f35b610641600480360381019061063c9190612611565b61147d565b60405161064e919061264d565b60405180910390f35b610671600480360381019061066c9190612611565b61150c565b60405161067e919061264d565b60405180910390f35b6106a1600480360381019061069c9190612611565b611552565b6040516106ae919061264d565b60405180910390f35b6106d160048036038101906106cc9190612611565b611590565b6040516106de919061264d565b60405180910390f35b61070160048036038101906106fc9190612611565b6115cc565b60405161070e919061264d565b60405180910390f35b610731600480360381019061072c9190612611565b611607565b60405161073e919061264d565b60405180910390f35b610761600480360381019061075c9190612611565b611646565b60405161076e919061264d565b60405180910390f35b610791600480360381019061078c9190612611565b611681565b60405161079e919061264d565b60405180910390f35b6107c160048036038101906107bc9190612611565b6116bc565b6040516107ce919061264d565b60405180910390f35b6107f160048036038101906107ec9190612611565b6116f7565b6040516107fe9190612876565b60405180910390f35b61080f6117a3565b60405161081c919061264d565b60405180910390f35b61083f600480360381019061083a9190612611565b6117c2565b60405161084c919061264d565b60405180910390f35b61086f600480360381019061086a9190612611565b6117fe565b60405161087c919061264d565b60405180910390f35b61089f600480360381019061089a9190612611565b61183a565b6040516108ac919061264d565b60405180910390f35b6108cf60048036038101906108ca9190612611565b611879565b6040516108dc919061264d565b60405180910390f35b6108ff60048036038101906108fa9190612611565b6118b4565b60405161090c919061264d565b60405180910390f35b61092f600480360381019061092a9190612611565b6118f2565b60405161093c919061264d565b60405180910390f35b61095f600480360381019061095a9190612611565b61192d565b60405161096c919061264d565b60405180910390f35b61098f600480360381019061098a9190612611565b611972565b60405161099c919061264d565b60405180910390f35b6109bf60048036038101906109ba91906127ae565b6119ae565b6040516109cc9190612876565b60405180910390f35b6109ef60048036038101906109ea91906127ae565b6119e0565b6040516109fc91906128b1565b60405180910390f35b610a0d611a0c565b604051610a1a919061264d565b60405180910390f35b610a3d6004803603810190610a389190612611565b611a48565b604051610a4a919061264d565b60405180910390f35b610a6d6004803603810190610a689190612611565b611a86565b604051610a7a919061264d565b60405180910390f35b610a9d6004803603810190610a989190612611565b611ac1565b604051610aaa919061264d565b60405180910390f35b610acd6004803603810190610ac89190612611565b611aff565b604051610ada919061264d565b60405180910390f35b610afd6004803603810190610af89190612611565b611b3b565b604051610b0a919061264d565b60405180910390f35b610b2d6004803603810190610b289190612611565b611b76565b604051610b3a919061264d565b60405180910390f35b610b5d6004803603810190610b589190612611565b611bb2565b604051610b6a919061264d565b60405180910390f35b610b8d6004803603810190610b889190612611565b611c0f565b604051610b9a919061264d565b60405180910390f35b610bbd6004803603810190610bb89190612611565b611c4e565b604051610bca919061264d565b60405180910390f35b610bed6004803603810190610be89190612611565b611c89565b604051610bfa919061264d565b60405180910390f35b610c1d6004803603810190610c1891906127ae565b611cd5565b604051610c2a9190612876565b60405180910390f35b610c4d6004803603810190610c489190612611565b611d43565b604051610c5a919061264d565b60405180910390f35b610c7d6004803603810190610c789190612611565b611d83565b604051610c8a919061264d565b60405180910390f35b610cad6004803603810190610ca891906127ae565b611dbe565b604051610cba9190612876565b60405180910390f35b610cdd6004803603810190610cd891906127ae565b611def565b604051610cea9190612876565b60405180910390f35b610d0d6004803603810190610d0891906127ae565b611e16565b604051610d1a919061290d565b60405180910390f35b610d2b611e98565b604051610d38919061264d565b60405180910390f35b610d5b6004803603810190610d569190612611565b611ee3565b604051610d68919061264d565b60405180910390f35b610d8b6004803603810190610d869190612611565b611f1e565b604051610d98919061264d565b60405180910390f35b610dbb6004803603810190610db69190612611565b611f5a565b604051610dc8919061264d565b60405180910390f35b610deb6004803603810190610de69190612988565b611f96565b604051610df8919061264d565b60405180910390f35b610e1b6004803603810190610e169190612611565b611fe4565b604051610e28919061264d565b60405180910390f35b610e4b6004803603810190610e469190612611565b61201f565b604051610e58919061264d565b60405180910390f35b610e7b6004803603810190610e769190612611565b61205a565b604051610e88919061264d565b60405180910390f35b610eab6004803603810190610ea69190612611565b612095565b604051610eb8919061264d565b60405180910390f35b610edb6004803603810190610ed69190612611565b6120d0565b604051610ee8919061264d565b60405180910390f35b610f0b6004803603810190610f069190612611565b612114565b604051610f18919061264d565b60405180910390f35b610f3b6004803603810190610f369190612611565b612150565b604051610f48919061264d565b60405180910390f35b610f6b6004803603810190610f669190612611565b61218b565b604051610f78919061264d565b60405180910390f35b610f9b6004803603810190610f969190612611565b6121c9565b604051610fa8919061264d565b60405180910390f35b610fcb6004803603810190610fc69190612611565b612206565b604051610fd8919061264d565b60405180910390f35b610ffb6004803603810190610ff69190612611565b612240565b604051611008919061264d565b60405180910390f35b61102b60048036038101906110269190612611565b61227c565b604051611038919061264d565b60405180910390f35b61105b60048036038101906110569190612611565b6122b8565b604051611068919061264d565b60405180910390f35b61108b60048036038101906110869190612611565b612313565b604051611098919061264d565b60405180910390f35b6110bb60048036038101906110b69190612611565b612355565b6040516110c8919061264d565b60405180910390f35b6110eb60048036038101906110e69190612611565b612391565b6040516110f8919061264d565b60405180910390f35b61111b60048036038101906111169190612611565b6123ce565b604051611128919061264d565b60405180910390f35b61114b600480360381019061114691906127ae565b612410565b6040516111589190612876565b60405180910390f35b61117b60048036038101906111769190612611565b61247f565b604051611188919061264d565b60405180910390f35b6111ab60048036038101906111a69190612611565b6124bb565b6040516111b8919061264d565b60405180910390f35b6111db60048036038101906111d69190612611565b6124f9565b6040516111e8919061264d565b60405180910390f35b61120b600480360381019061120691906127ae565b612538565b6040516112189190612a10565b60405180910390f35b61123b60048036038101906112369190612611565b61256a565b604051611248919061264d565b60405180910390f35b600061125b6117a3565b50600065deadbeef003690506000805b848110156112815736915060018101905061126b565b505080915050919050565b60006112966117a3565b50600065deadbeef003290506000805b848110156112bc573291506001810190506112a6565b505080915050919050565b60006112d16117a3565b50600065deadbeef0052905060005b838110156112f757816000526001810190506112e0565b5080915050919050565b60606000600890506040828451602086016000855af18061132157600080fd5b5050919050565b60006113326117a3565b50600065deadbeef0001905060005b8381101561135a57600082019150600181019050611341565b5080915050919050565b60008054905090565b60006113776117a3565b50600065deadbeef0017905060005b8381101561139f57600082179150600181019050611386565b5080915050919050565b60006113b36117a3565b50600065deadbeef003490506000805b848110156113d9573491506001810190506113c3565b505080915050919050565b60006113ee6117a3565b50600065deadbeef0006905060005b83811015611435577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff820691506001810190506113fd565b5080915050919050565b60006114496117a3565b50600065deadbeef001390506000805b8481101561147257600183139150600181019050611459565b505080915050919050565b60006114876117a3565b50600065deadbeef002090507fffffffff000000000000000000000000000000000000000000000000000000006000526000805b848110156114d557600460002091506001810190506114bb565b507f29045a592007d0c246ef02c2223570da9522d0cf0f73282c79a1bc8f0bb2c238811461150257600091505b5080915050919050565b60006115166117a3565b50600065deadbeef00a490508060105260005b83811015611548576004600360028360066010a4600181019050611529565b5080915050919050565b600061155c6117a3565b50600065deadbeef001a90506000805b84811015611585578260001a915060018101905061156c565b505080915050919050565b600061159a6117a3565b50600065deadbeef001b905060005b838110156115c2578160001b91506001810190506115a9565b5080915050919050565b60006115d66117a3565b50600065deadbeef004290506000805b848110156115fc574291506001810190506115e6565b505080915050919050565b60006116116117a3565b50600065deadbeef0031905060003060005b8581101561163a5781319250600181019050611623565b50505080915050919050565b60006116506117a3565b50600065deadbeef004890506000805b8481101561167657489150600181019050611660565b505080915050919050565b600061168b6117a3565b50600065deadbeef003d90506000805b848110156116b1573d915060018101905061169b565b505080915050919050565b60006116c66117a3565b50600065deadbeef004390506000805b848110156116ec574391506001810190506116d6565b505080915050919050565b6002818154811061170757600080fd5b90600052602060002001600091509050805461172290612a5a565b80601f016020809104026020016040519081016040528092919081815260200182805461174e90612a5a565b801561179b5780601f106117705761010080835404028352916020019161179b565b820191906000526020600020905b81548152906001019060200180831161177e57829003601f168201915b505050505081565b600060016000546117b49190612aba565b600081905550600054905090565b60006117cc6117a3565b50600065deadbeef0004905060005b838110156117f4576001820491506001810190506117db565b5080915050919050565b60006118086117a3565b50600065deadbeef0037905060005b8381101561183057602060008037600181019050611817565b5080915050919050565b60006118446117a3565b50600065deadbeef00a090508060105260005b8381101561186f5760066010a0600181019050611857565b5080915050919050565b60006118836117a3565b50600065deadbeef003390506000805b848110156118a957339150600181019050611893565b505080915050919050565b60006118be6117a3565b50600065deadbeef0053905060005b838110156118e85763deadbeef6000526001810190506118cd565b5080915050919050565b60006118fc6117a3565b50600065deadbeef003a90506000805b84811015611922573a915060018101905061190c565b505080915050919050565b60006119376117a3565b50600065deadbeef0051905060008160005260005b8481101561196457600051915060018101905061194c565b508091505080915050919050565b600061197c6117a3565b50600065deadbeef001d905060005b838110156119a4578160001d915060018101905061198b565b5080915050919050565b606060006005905060208301835160405160208183856000885af1806119d357600080fd5b8195505050505050919050565b600080600290506020830183518360208183856000885af180611a0257600080fd5b5050505050919050565b6000611a166117a3565b505b6103e85a1115611a40576001806000828254611a349190612aba565b92505081905550611a18565b600154905090565b6000611a526117a3565b50600065deadbeef001090506000805b84811015611a7b57826001109150600181019050611a62565b505080915050919050565b6000611a906117a3565b50600065deadbeef004490506000805b84811015611ab657449150600181019050611aa0565b505080915050919050565b6000611acb6117a3565b50600065deadbeef001190506000805b84811015611af457600183119150600181019050611adb565b505080915050919050565b6000611b096117a3565b50600065deadbeef003e905060005b83811015611b315760206000803e600181019050611b18565b5080915050919050565b6000611b456117a3565b50600065deadbeef004590506000805b84811015611b6b57459150600181019050611b55565b505080915050919050565b6000611b806117a3565b50600065deadbeef0002905060005b83811015611ba857600182029150600181019050611b8f565b5080915050919050565b6000611bbc6117a3565b50600065deadbeef0008905060005b83811015611c05577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff600083089150600181019050611bcb565b5080915050919050565b6000611c196117a3565b50600065deadbeef005490508060005560005b83811015611c44576000549150600181019050611c2c565b5080915050919050565b6000611c586117a3565b50600065deadbeef005a90506000805b84811015611c7e575a9150600181019050611c68565b505080915050919050565b6000611c936117a3565b50600065deadbeef0019905060005b83811015611cb95781199150600181019050611ca2565b5065deadbeef00198114611ccc57801990505b80915050919050565b606080825114611d1a576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401611d1190612b4b565b60405180910390fd5b60006007905060208301835160408482846000875af180611d3a57600080fd5b50505050919050565b6000611d4d6117a3565b50600065deadbeef00a190508060105260005b83811015611d79578060066010a1600181019050611d60565b5080915050919050565b6000611d8d6117a3565b50600065deadbeef0016905060005b83811015611db4578182169150600181019050611d9c565b5080915050919050565b6060600060049050602083018351604051818183856000885af180611de257600080fd5b8195505050505050919050565b60606000600890506040828451602086016000855af180611e0f57600080fd5b5050919050565b60006080825114611e5c576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401611e5390612bb7565b60405180910390fd5b600060019050602083016020810151601f1a602082015260206040516080836000865af180611e8a57600080fd5b604051519350505050919050565b6000611ea26117a3565b505b6103e85a1115611edb576001806000828254611ec09190612aba565b9250508190555043600154611ed59190612c06565b50611ea4565b600154905090565b6000611eed6117a3565b50600065deadbeef004690506000805b84811015611f1357469150600181019050611efd565b505080915050919050565b6000611f286117a3565b50600065deadbeef0005905060005b83811015611f5057600182059150600181019050611f37565b5080915050919050565b6000611f646117a3565b50600065deadbeef0039905060005b83811015611f8c57602060008039600181019050611f73565b5080915050919050565b60006002838390918060018154018082558091505060019003906000526020600020016000909192909192909192909192509182611fd5929190612dee565b50600280549050905092915050565b6000611fee6117a3565b50600065deadbeef005990506000805b8481101561201457599150600181019050611ffe565b505080915050919050565b60006120296117a3565b50600065deadbeef003890506000805b8481101561204f57389150600181019050612039565b505080915050919050565b60006120646117a3565b50600065deadbeef004190506000805b8481101561208a57419150600181019050612074565b505080915050919050565b600061209f6117a3565b50600065deadbeef003090506000805b848110156120c5573091506001810190506120af565b505080915050919050565b60006120da6117a3565b50600065deadbeef00a390508060105260005b8381101561210a57600360028260066010a36001810190506120ed565b5080915050919050565b600061211e6117a3565b50600065deadbeef000b905060005b83811015612146578160200b915060018101905061212d565b5080915050919050565b600061215a6117a3565b50600065deadbeef004790506000805b848110156121805747915060018101905061216a565b505080915050919050565b60006121956117a3565b50600065deadbeef001c90506000805b848110156121be578260001c92506001810190506121a5565b505080915050919050565b60006121d36117a3565b50600065deadbeef003590506000805b848110156121fb5760003591506001810190506121e3565b505080915050919050565b60006122106117a3565b50600065deadbeef0055905060005b83811015612236578160005560018101905061221f565b5080915050919050565b600061224a6117a3565b50600065deadbeef0018905060005b8381101561227257600082189150600181019050612259565b5080915050919050565b60006122866117a3565b50600065deadbeef0003905060005b838110156122ae57600082039150600181019050612295565b5080915050919050565b60006122c26117a3565b50600065deadbeef0007905060005b83811015612309577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff820791506001810190506122d1565b5080915050919050565b600061231d6117a3565b50600065deadbeef00a290508060105260005b8381101561234b5760028160066010a2600181019050612330565b5080915050919050565b600061235f6117a3565b50600065deadbeef000a905060005b83811015612387576001820a915060018101905061236e565b5080915050919050565b600061239b6117a3565b50600065deadbeef001490506000805b848110156123c35782831491506001810190506123ab565b505080915050919050565b60006123d86117a3565b50600065deadbeef0040905060006001430360005b8581101561240457814092506001810190506123ed565b50505080915050919050565b60606080825114612456576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161244d90612b4b565b60405180910390fd5b60006006905060208301835160408482846000875af18061247657600080fd5b50505050919050565b60006124896117a3565b50600065deadbeef001590506000805b848110156124b05782159150600181019050612499565b505080915050919050565b60006124c56117a3565b50600065deadbeef001290506000805b848110156124ee578260011291506001810190506124d5565b505080915050919050565b60006125036117a3565b50600065deadbeef003b905060003060005b8581101561252c57813b9250600181019050612515565b50505080915050919050565b6000806003905060208301835160405160148183856000885af18061255c57600080fd5b815195505050505050919050565b60006125746117a3565b50600065deadbeef0009905060005b838110156125bd577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff600183099150600181019050612583565b5080915050919050565b6000604051905090565b600080fd5b600080fd5b6000819050919050565b6125ee816125db565b81146125f957600080fd5b50565b60008135905061260b816125e5565b92915050565b600060208284031215612627576126266125d1565b5b6000612635848285016125fc565b91505092915050565b612647816125db565b82525050565b6000602082019050612662600083018461263e565b92915050565b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b6126bb82612672565b810181811067ffffffffffffffff821117156126da576126d9612683565b5b80604052505050565b60006126ed6125c7565b90506126f982826126b2565b919050565b600067ffffffffffffffff82111561271957612718612683565b5b61272282612672565b9050602081019050919050565b82818337600083830152505050565b600061275161274c846126fe565b6126e3565b90508281526020810184848401111561276d5761276c61266d565b5b61277884828561272f565b509392505050565b600082601f83011261279557612794612668565b5b81356127a584826020860161273e565b91505092915050565b6000602082840312156127c4576127c36125d1565b5b600082013567ffffffffffffffff8111156127e2576127e16125d6565b5b6127ee84828501612780565b91505092915050565b600081519050919050565b600082825260208201905092915050565b60005b83811015612831578082015181840152602081019050612816565b60008484015250505050565b6000612848826127f7565b6128528185612802565b9350612862818560208601612813565b61286b81612672565b840191505092915050565b60006020820190508181036000830152612890818461283d565b905092915050565b6000819050919050565b6128ab81612898565b82525050565b60006020820190506128c660008301846128a2565b92915050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b60006128f7826128cc565b9050919050565b612907816128ec565b82525050565b600060208201905061292260008301846128fe565b92915050565b600080fd5b600080fd5b60008083601f84011261294857612947612668565b5b8235905067ffffffffffffffff81111561296557612964612928565b5b6020830191508360018202830111156129815761298061292d565b5b9250929050565b6000806020838503121561299f5761299e6125d1565b5b600083013567ffffffffffffffff8111156129bd576129bc6125d6565b5b6129c985828601612932565b92509250509250929050565b60007fffffffffffffffffffffffffffffffffffffffff00000000000000000000000082169050919050565b612a0a816129d5565b82525050565b6000602082019050612a256000830184612a01565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680612a7257607f821691505b602082108103612a8557612a84612a2b565b5b50919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000612ac5826125db565b9150612ad0836125db565b9250828201905080821115612ae857612ae7612a8b565b5b92915050565b600082825260208201905092915050565b7f496e76616c696420696e707574206c656e677468000000000000000000000000600082015250565b6000612b35601483612aee565b9150612b4082612aff565b602082019050919050565b60006020820190508181036000830152612b6481612b28565b9050919050565b7f496e76616c696420696e7075742064617461206c656e6774682e000000000000600082015250565b6000612ba1601a83612aee565b9150612bac82612b6b565b602082019050919050565b60006020820190508181036000830152612bd081612b94565b9050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601260045260246000fd5b6000612c11826125db565b9150612c1c836125db565b925082612c2c57612c2b612bd7565b5b828206905092915050565b600082905092915050565b60008190508160005260206000209050919050565b60006020601f8301049050919050565b600082821b905092915050565b600060088302612ca47fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82612c67565b612cae8683612c67565b95508019841693508086168417925050509392505050565b6000819050919050565b6000612ceb612ce6612ce1846125db565b612cc6565b6125db565b9050919050565b6000819050919050565b612d0583612cd0565b612d19612d1182612cf2565b848454612c74565b825550505050565b600090565b612d2e612d21565b612d39818484612cfc565b505050565b5b81811015612d5d57612d52600082612d26565b600181019050612d3f565b5050565b601f821115612da257612d7381612c42565b612d7c84612c57565b81016020851015612d8b578190505b612d9f612d9785612c57565b830182612d3e565b50505b505050565b600082821c905092915050565b6000612dc560001984600802612da7565b1980831691505092915050565b6000612dde8383612db4565b9150826002028217905092915050565b612df88383612c37565b67ffffffffffffffff811115612e1157612e10612683565b5b612e1b8254612a5a565b612e26828285612d61565b6000601f831160018114612e555760008415612e43578287013590505b612e4d8582612dd2565b865550612eb5565b601f198416612e6386612c42565b60005b82811015612e8b57848901358255600182019150602085019450602081019050612e66565b86831015612ea85784890135612ea4601f891682612db4565b8355505b6001600288020188555050505b5050505050505056fea26469706673582212203124213488c2f1fca5968787f0c3e96fba8469129a80798e11ee752903b4bfdc64736f6c634300081300330058200252130bf561a2ad9468cb2919d5ff2cda5c508338aaa5a12ee06e43acf1fa335820baaaaaadbaadf00dbad22222baddcafecafeb0bab0bababebeefbabec00010ff005820020b976be9384d1bb7a9ba3c6f92f3dffbefb6aaa4a07626c32489cd66e20473581f0ff1ce00bab10c1badb0028badf00dabadbabeb105f00db16b00b50b00b1350219080400582002b0c6948a275349ae45a06aad66a8bd65ac18074615d53676c09b67809099e0410200582002c72455231bf4548b418278aebda259695706344fedffefb40d8218532f72125820deadbeafdeadbeefdeadc0dedeaddeaddeadd00ddeadfa11dead10ccdeadfeed02190c00005820027eff41a0dce30a6e5bdeb23d1bbb96709facaf0abff8949749f89c697a7edd5820cafebabecafed00dcefaedfe0d15ea5edabbad00dead2baddeadbaaddeadbabe034d6a690768a0ea387b759e0bef01ee064b5d04cf830ff8fa74104e5dbeafab090219a000005820025787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace410900582002a69471df6e569a3d0da24943b5a847e21da73a0d58b0a25836633793cbf2dc5820deadbeafdeadbeefdeadc0dedeaddeaddeadd00ddeadfa11dead10ccdeadfeed00582002ee6d38ad948303a0117a3e3deee4d912b62481681bd892442a7d720eee5d2c581f0ff1ce000000000000000000000000000000000000000000000000000000080219044100582103780bd76754cd8bdf6ebbcf526b1e9c300885e157b72e09c4f68214c616f7bd30418100582103700f56bdfffe5f336e60cc5d9ad093591a43a048d8c82013fa9eb71ae98739905820baaaaaadbaadf00dbad22222baddcafecafeb0bab0bababebeefbabec00010ff00582103f64f60661322b36af17ffae1d83bdb731d45dce1596efffa3ccfc42c4aa182a05820b105f00db16b00b50b00b135baaaaaadbaadf00dbad22222baddcafecafeb0ba0334f927d8cb7dd37b23b0e1760984f38c0654cade533e23af873c94318811099903f399c14a1aca218d9f65fde0fede5584dd350446a9b85edb2531cd8ca793008f00582002b7834d611e25670b584f73a3e810d0a47c773fe173fc6975449e876b0a6a70581f0ff1ce00bab10c00000000000000000000000000000000000000000000001003eea55a2063723ec5f83b1bc2fd4a14edd99b58afad68631b87dc0ac06cf12a3500582002ca152920095f2fe7984b9ce1a725c3bc9436952ed17113f5fc7b7b613c401d420201021902c003316c463a8777740576aedfdd3d859851b8cc455ec2c3c2fe2b235e102e59eeb6005821035126a4d711f2dd98aa7df46b100c291503dddb43ad8180ae07f600704524a9d0414100582103605e486497dbb470ce04bc6cd8d6aa1cc0fa707511d6bcc61d0dbc85551736605820cafebabecafed00dcefaedfe0d15ea5edabbad00dead2baddeadbaaddeadbabe0219df770558210336d6fadc19b5ec9189ae65683241081f7c772ec596ea1facb9daef2a139663700701192ef40219fd73", root_str: "5d18708fa7f7c751cf1528a5dd7ce11911a4eaeaef2b06f0c3e6e0cbce303e19" };

type ProcessCompactPrestateFn = fn(TrieCompact) -> CompactParsingResult<ProcessedCompactOutput>;

pub(crate) struct TestProtocolInputAndRoot {
    pub(crate) byte_str: &'static str,
    pub(crate) root_str: &'static str,
}

impl TestProtocolInputAndRoot {
    pub(crate) fn parse_and_check_hash_matches(self) {
        self.parse_and_check_hash_matches_common(process_compact_prestate);
    }

    pub(crate) fn parse_and_check_hash_matches_with_debug(self) {
        self.parse_and_check_hash_matches_common(process_compact_prestate_debug);
    }

    fn parse_and_check_hash_matches_common(
        self,
        process_compact_prestate_f: ProcessCompactPrestateFn,
    ) {
        let protocol_bytes = hex::decode(self.byte_str).unwrap();
        let expected_hash = TrieRootHash::from_slice(&hex::decode(self.root_str).unwrap());

        let out = match process_compact_prestate_f(TrieCompact(protocol_bytes)) {
            Ok(x) => x,
            Err(err) => panic!("{}", err.to_string()),
        };
        let trie_hash = out.witness_out.tries.state.hash();

        assert!(out.header.version_is_compatible(1));
        assert_eq!(trie_hash, expected_hash);
    }
}
