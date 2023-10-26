use crate::cache::{Cache, FileSystemCache};
use crate::evm::contract_utils::ABIConfig;
use heimdall_core::decompile::{decompile, DecompilerArgsBuilder, out::abi::ABIStructure};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use tracing::debug;

pub fn fetch_abi_heimdall(bytecode: String) -> Vec<ABIConfig> {
    let mut hasher = DefaultHasher::new();
    bytecode.hash(&mut hasher);
    let bytecode_hash = hasher.finish();
    let cache_key = format!("{}.json", bytecode_hash);
    let cache = FileSystemCache::new("cache/heimdall");
    match cache.load(cache_key.as_str()) {
        Ok(res) => {
            debug!("using cached result of decompiling contract");
            return serde_json::from_str(res.as_str()).unwrap();
        }
        Err(_) => {}
    }
    let heimdall_result = decompile_with_bytecode(bytecode).expect("unable to decompile contract");
    let mut result = vec![];
    for heimdall_abi in heimdall_result {
        match heimdall_abi {
            ABIStructure::Function(func) => {
                let mut inputs = vec![];
                for input in func.inputs {
                    let ty = input.type_;
                    if ty == "bytes" {
                        inputs.push("unknown".to_string());
                    } else {
                        inputs.push(ty);
                    }
                }

                let name = func.name.replace("Unresolved_", "");
                let mut abi_config = ABIConfig {
                    abi: format!("({})", inputs.join(",")),
                    function: [0; 4],
                    function_name: name.clone(),
                    is_static: func.state_mutability == "view",
                    is_payable: func.state_mutability == "payable",
                    is_constructor: false,
                };
                abi_config
                    .function
                    .copy_from_slice(hex::decode(name).unwrap().as_slice());
                result.push(abi_config)
            }
            _ => {
                continue;
            }
        }
    }
    FileSystemCache::new("cache/heimdall")
        .save(
            cache_key.as_str(),
            serde_json::to_string(&result).unwrap().as_str(),
        )
        .expect("unable to save cache");
    result
}

fn decompile_with_bytecode(contract_bytecode: String) -> Result<Vec<ABIStructure>, Box<dyn Error>>{
    let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

    let args = DecompilerArgsBuilder::new()
        .target(contract_bytecode)
        .build()?;

    let res = rt.block_on(decompile(args))?;
    res.abi.ok_or("unable to decompile contract".into())
}

mod tests {
    use super::*;

    #[test]
    fn test_heimdall() {
        debug!("{:?}", fetch_abi_heimdall(
            "0x6080604052600436106101395760003560e01c8063715018a6116100ab578063b6fccf8a1161006f578063b6fccf8a1461035e578063db006a751461037e578063dd62ed3e1461039e578063eb37acfc146103d6578063f2fde38b146103f6578063fec362351461041657600080fd5b8063715018a6146102e15780638da5cb5b146102f657806395d89b4114610314578063a9059cbb14610329578063b69ef8a81461034957600080fd5b80632f34d282116100fd5780632f34d282146101f057806330d5baea14610210578063313ce567146102305780634d95cad91461025c57806351cff8d91461029457806370a08231146102b457600080fd5b806306fdde031461014e578063095ea7b31461017957806318160ddd146101a95780631d9053e0146101c857806323b872dd146101d057600080fd5b366101495761014733610436565b005b600080fd5b34801561015a57600080fd5b506101636104e2565b60405161017091906115d2565b60405180910390f35b34801561018557600080fd5b5061019961019436600461161a565b610570565b6040519015158152602001610170565b3480156101b557600080fd5b506005545b604051908152602001610170565b6101476105dc565b3480156101dc57600080fd5b506101996101eb366004611646565b6105f9565b3480156101fc57600080fd5b5061014761020b366004611687565b61077d565b34801561021c57600080fd5b5061014761022b3660046116a4565b6107f5565b34801561023c57600080fd5b5060045461024a9060ff1681565b60405160ff9091168152602001610170565b34801561026857600080fd5b5060075461027c906001600160a01b031681565b6040516001600160a01b039091168152602001610170565b3480156102a057600080fd5b506101476102af366004611687565b6108cd565b3480156102c057600080fd5b506101ba6102cf366004611687565b60096020526000908152604090205481565b3480156102ed57600080fd5b506101476109a9565b34801561030257600080fd5b506001546001600160a01b031661027c565b34801561032057600080fd5b506101636109bb565b34801561033557600080fd5b5061019961034436600461161a565b6109c8565b34801561035557600080fd5b506006546101ba565b34801561036a57600080fd5b5060085461027c906001600160a01b031681565b34801561038a57600080fd5b506101476103993660046116a4565b6109dc565b3480156103aa57600080fd5b506101ba6103b93660046116bd565b600a60209081526000928352604080842090915290825290205481565b3480156103e257600080fd5b506101476103f13660046116a4565b610b04565b34801561040257600080fd5b50610147610411366004611687565b610f3e565b34801561042257600080fd5b506101476104313660046116a4565b610fb4565b6007546001600160a01b03828116911614610479576001600160a01b0381166000908152600960205260408120805434929061047390849061170c565b90915550505b346005600082825461048b919061170c565b9091555061049c9050346001610fcf565b806001600160a01b03167fe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c346040516104d791815260200190565b60405180910390a250565b600280546104ef90611724565b80601f016020809104026020016040519081016040528092919081815260200182805461051b90611724565b80156105685780601f1061053d57610100808354040283529160200191610568565b820191906000526020600020905b81548152906001019060200180831161054b57829003601f168201915b505050505081565b336000818152600a602090815260408083206001600160a01b038716808552925280832085905551919290917f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925906105cb9086815260200190565b60405180910390a350600192915050565b6105e4611011565b6105ed33610436565b6105f76001600055565b565b6001600160a01b03831660009081526009602052604081205482111561061e57600080fd5b6001600160a01b038416331480159061065c57506001600160a01b0384166000908152600a6020908152604080832033845290915290205460001914155b156106ca576001600160a01b0384166000908152600a6020908152604080832033845290915290205482111561069157600080fd5b6001600160a01b0384166000908152600a60209081526040808320338452909152812080548492906106c490849061175f565b90915550505b6001600160a01b038416600090815260096020526040812080548492906106f290849061175f565b90915550506001600160a01b0383166000908152600960205260408120805484929061071f90849061170c565b92505081905550826001600160a01b0316846001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8460405161076b91815260200190565b60405180910390a35060019392505050565b61078561106b565b6001600160a01b0381166107d35760405162461bcd60e51b815260206004820152601060248201526f4e4f4e205a45524f204144445245535360801b60448201526064015b60405180910390fd5b600880546001600160a01b0319166001600160a01b0392909216919091179055565b6107fd611011565b3360009081526009602052604090205481111561082c5760405162461bcd60e51b81526004016107ca90611776565b336000908152600960205260408120805483929061084b90849061175f565b925050819055508060056000828254610864919061175f565b9091555050600754610880906001600160a01b031633836110c5565b61088b816000610fcf565b60405181815233907f7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b659060200160405180910390a26108ca6001600055565b50565b6108d561106b565b63637a874042101561091c5760405162461bcd60e51b815260206004820152601060248201526f1413d3d3081393d5081156141254915160821b60448201526064016107ca565b6001600160a01b038116610934576108ca33476111f6565b6040516370a0823160e01b81523060048201526108ca90829033906001600160a01b038316906370a0823190602401602060405180830381865afa158015610980573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906109a491906117a1565b6110c5565b6109b161106b565b6105f760006112d5565b600380546104ef90611724565b60006109d53384846105f9565b9392505050565b6109e4611011565b33600090815260096020526040902054811115610a135760405162461bcd60e51b81526004016107ca90611776565b3360009081526009602052604081208054839290610a3290849061175f565b925050819055508060056000828254610a4b919061175f565b9091555050604051600090339083908381818185875af1925050503d8060008114610a92576040519150601f19603f3d011682016040523d82523d6000602084013e610a97565b606091505b5050905080610ab957604051633204506f60e01b815260040160405180910390fd5b610ac4826000610fcf565b60405182815233907f7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b659060200160405180910390a2506108ca6001600055565b610b0c611011565b6008546001600160a01b0316610b4d5760405162461bcd60e51b81526020600482015260066024820152650534554204c560d41b60448201526064016107ca565b6008546040516370a0823160e01b815233600482015282916001600160a01b0316906370a0823190602401602060405180830381865afa158015610b95573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610bb991906117a1565b1015610bd75760405162461bcd60e51b81526004016107ca90611776565b6008546040516370a0823160e01b81523060048201526000916001600160a01b0316906370a0823190602401602060405180830381865afa158015610c20573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610c4491906117a1565b6008546040516323b872dd60e01b8152336004820152306024820152604481018590529192506001600160a01b0316906323b872dd906064016020604051808303816000875af1158015610c9c573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610cc091906117ba565b506008546040516370a0823160e01b81523060048201526000916001600160a01b0316906370a0823190602401602060405180830381865afa158015610d0a573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610d2e91906117a1565b905082610d3b838361175f565b1015610d7f5760405162461bcd60e51b81526020600482015260136024820152720a8a4829ca68c8aa4409c9ea8408a9c9eaa8e9606b1b60448201526064016107ca565b6008546040516323b872dd60e01b81523360048201526001600160a01b039091166024820181905260448201859052906323b872dd906064016020604051808303816000875af1158015610dd7573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610dfb91906117ba565b5060085460405163226bf2d160e21b815233600482015260009182916001600160a01b03909116906389afcb449060240160408051808303816000875af1158015610e4a573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610e6e91906117dc565b91509150306001600160a01b0316600860009054906101000a90046001600160a01b03166001600160a01b0316630dfe16816040518163ffffffff1660e01b8152600401602060405180830381865afa158015610ecf573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610ef39190611800565b6001600160a01b03161415610f1b57610f0b81611327565b610f1533836109c8565b50610f30565b610f2482611327565b610f2e33826109c8565b505b505050506108ca6001600055565b610f4661106b565b6001600160a01b038116610fab5760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b60648201526084016107ca565b6108ca816112d5565b610fbc611011565b610fc581611327565b6108ca6001600055565b80610ff1578160066000828254610fe6919061175f565b92505081905561100a565b8160066000828254611003919061170c565b9250508190555b6006555050565b600260005414156110645760405162461bcd60e51b815260206004820152601f60248201527f5265656e7472616e637947756172643a207265656e7472616e742063616c6c0060448201526064016107ca565b6002600055565b6001546001600160a01b031633146105f75760405162461bcd60e51b815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e657260448201526064016107ca565b604080516001600160a01b038481166024830152604480830185905283518084039091018152606490920183526020820180516001600160e01b031663a9059cbb60e01b1790529151600092839290871691611121919061181d565b6000604051808303816000865af19150503d806000811461115e576040519150601f19603f3d011682016040523d82523d6000602084013e611163565b606091505b509150915081801561118d57508051158061118d57508080602001905181019061118d91906117ba565b6111ef5760405162461bcd60e51b815260206004820152602d60248201527f5472616e7366657248656c7065723a3a736166655472616e736665723a20747260448201526c185b9cd9995c8819985a5b1959609a1b60648201526084016107ca565b5050505050565b604080516000808252602082019092526001600160a01b038416908390604051611220919061181d565b60006040518083038185875af1925050503d806000811461125d576040519150601f19603f3d011682016040523d82523d6000602084013e611262565b606091505b50509050806112d05760405162461bcd60e51b815260206004820152603460248201527f5472616e7366657248656c7065723a3a736166655472616e736665724554483a60448201527308115512081d1c985b9cd9995c8819985a5b195960621b60648201526084016107ca565b505050565b600180546001600160a01b038381166001600160a01b0319831681179093556040519116919082907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a35050565b6007546040516370a0823160e01b815233600482015282916001600160a01b0316906370a0823190602401602060405180830381865afa15801561136f573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061139391906117a1565b10156113b15760405162461bcd60e51b81526004016107ca90611776565b6007546040516370a0823160e01b81523060048201526000916001600160a01b0316906370a0823190602401602060405180830381865afa1580156113fa573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061141e91906117a1565b6007546040516323b872dd60e01b8152336004820152306024820152604481018590529192506001600160a01b0316906323b872dd906064016020604051808303816000875af1158015611476573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061149a91906117ba565b506007546040516370a0823160e01b81523060048201526000916001600160a01b0316906370a0823190602401602060405180830381865afa1580156114e4573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061150891906117a1565b905082611515838361175f565b10156115595760405162461bcd60e51b81526020600482015260136024820152720a8a4829ca68c8aa4409c9ea8408a9c9eaa8e9606b1b60448201526064016107ca565b336000908152600960205260408120805485929061157890849061170c565b925050819055508260056000828254611591919061170c565b909155506112d09050836001610fcf565b60005b838110156115bd5781810151838201526020016115a5565b838111156115cc576000848401525b50505050565b60208152600082518060208401526115f18160408501602087016115a2565b601f01601f19169190910160400192915050565b6001600160a01b03811681146108ca57600080fd5b6000806040838503121561162d57600080fd5b823561163881611605565b946020939093013593505050565b60008060006060848603121561165b57600080fd5b833561166681611605565b9250602084013561167681611605565b929592945050506040919091013590565b60006020828403121561169957600080fd5b81356109d581611605565b6000602082840312156116b657600080fd5b5035919050565b600080604083850312156116d057600080fd5b82356116db81611605565b915060208301356116eb81611605565b809150509250929050565b634e487b7160e01b600052601160045260246000fd5b6000821982111561171f5761171f6116f6565b500190565b600181811c9082168061173857607f821691505b6020821081141561175957634e487b7160e01b600052602260045260246000fd5b50919050565b600082821015611771576117716116f6565b500390565b6020808252601190820152704e4f20454e4f5547482042414c414e434560781b604082015260600190565b6000602082840312156117b357600080fd5b5051919050565b6000602082840312156117cc57600080fd5b815180151581146109d557600080fd5b600080604083850312156117ef57600080fd5b505080516020909101519092909150565b60006020828403121561181257600080fd5b81516109d581611605565b6000825161182f8184602087016115a2565b919091019291505056fea2646970667358221220b9216a219819fff3398750dcd8547a3d24f16c386122922294dc501a8b0d549864736f6c634300080a0033".to_string(),
        ))
    }
}
