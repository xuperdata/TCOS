从[《超级链SDK结构》](./超级链SDK结构.pdf)可以了解当前SDK的架构。

本设计主要解决在TEE内部发起可信交易的问题，打造超级链的**可信数据上链ORACLE**。

SDK未来在3CO项目中会被TEE外面和TEE内部使用到，因此需要保证同一套代码，但是支持签名部分在可以发生在TEE内部， 所以整体SDK分为3个部分 [TEESDK](https://github.com/xuperdata/teesdk), [XuperSDK-Crypto](https://github.com/duanbing/xchain-rust-crypto) 以及XuperSDK-RPC

因此整体的拆分思路是：

```
      
   -->CAPI  |  GO      ---------------------    
  |   ------------    |											|				 -----
  |   XuperSDK-RPC  ------> XuperSDK-Crypto	|     ->| KMS |  Trusted World
  |Ocall+FFI    			|	 Ecall							| 		|	 -----
  |   ---------------- 											|			|		  
  |  | 								 				TEESDK -------------      
   --|- MesaTEE-FNS		 											|   
     --------------------------------------- 	Trusted World								

```

功能解释：

* XuperSDK-RPC:  组装交易，发送交易
* XuperSDK-Crypto： 通过Hash生成TXID，交易签名
* TEESDK: 负责跟KMS通信，加密交易的敏感字段；

在解释之前，先增加几个定义：

```
数据标识（Data ID）:  全局以为的代表当前数据的ID，可以是UUID或者编号。
数据存证信息(Data Credential): (Data ID, 加密meta, 描述信息, 发布时间，发布方信息, 签名) 
操作存证信息(Ops Credential): (TaskID, 操作人，操作日志hash，操作日志密文，操作时间，前一个操作ID, 签名)
```



核心操作流程如下（包括输入数据，输出数据以及计算过程日志存证）

1. MesaTEE-FNS调用对应的function计算完成之后，获得计算的结果文件，计算结果的存证信息(DC).  然后调用

XuperSDK-RPC开始**组装交易**；其中组装交易主要包含如下几个子流程：

> a. GenPreExecRequest
>
> b. GenComplianceCheckTx
>
> c. 

2. 



### 详细接口

#### XuperSDK-RPC

#### XuperSDK-Crypto





### 参考

1. 