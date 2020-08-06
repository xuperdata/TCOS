# TCOS设-双向认证

基本原理是在occlum创建process之前完成双向认证。



编译：

```
git clone https://github.com/occlum/occlum
cd occlum
git checkout 0.13.1
git am ${TCOS DIR}/0001-add-mutual-att.patch
cp -r ${TCOS DIR}/trusted-mesatee-sdk . 

docker run --net=host -it --device /dev/sgx/enclave   -v $PWD:/occlum occlum/occlum:0.13.1-ubuntu18.04 bash
cd /occlum 
make
make install 
```



执行：

1. 复制如下配置文件文件到 context目录

```
enclave_info.toml
runtime.config.toml
auditors/
```



2. 设置环境变量

```
export IAS_SPID=xxx
export IAS_KEY=xx
```

3. 执行

```
occum run /bin/xxx 
```

