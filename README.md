## PubChemsDownload

### 使用

```bash
pub_chems_data 0.1.0

USAGE:
    pub_chem_download [FLAGS] [OPTIONS]

FLAGS:
        --enable-db        使用db缓存结果数据,主要是用于404结果缓存
    -f, --enable-filter    开启过滤任务, 默认是下载, 开启后关闭下载专注于数据过滤
    -h, --help             Prints help information
        --no-update        过滤任务时, 不去更新已在数据库中的数据
    -v, --version          显示版本

OPTIONS:
    -p, --data_path <data-path>        筛选数据路径 [default: data]
    -b, --block <download-start>       下载任务起始cid [default: 0]
    -n, --filter-name <filter-name>    过滤任务标签 [default: ]
    -j <jobs>                          并行任务数 [default: 1]
    -s, --sql <sql>                    mongodb 服务地址 [default: 192.168.2.25:27017]
```

### 功能

*  根据`cid`从`pubChem`上下载`json`格式文件

```
./pub_chems_data -b 1 -j 8  # 下载`cid` 在`1000000-2000000`之间的文件, 开启并行数为8
```



*  筛选下载的`json`文件内容到数据库

```
./pub_chems_data -f  -p data # 开启筛选任务, 选择数据目录在`data`下
```