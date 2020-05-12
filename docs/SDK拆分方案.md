从[《超级链SDK结构》](./超级链SDK结构.pdf)可以了解当前SDK的架构。

SDK未来在3CO项目中会被TEE外面和TEE内部使用到，因此需要保证同一套代码，但是支持签名部分在可以发生在TEE内部， 所以整体SDK分为3个部分 TEESDK, XuperSDK-Crypto 以及XuperSDK-RPC

因此整体的拆分思路是：









