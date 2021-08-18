use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "pub_chems_data")]
pub struct Opt {
    #[structopt(short = "v", long, help = "显示版本")]
    pub version: bool,

    #[structopt(long = "enable-db", help = "使用db缓存结果数据,主要是用于404结果缓存")]
    pub enable_db: bool,

    #[structopt(short = "j", help = "并行任务数", default_value = "1")]
    pub jobs: usize,

    #[structopt(
        long = "block",
        short = "b",
        help = "下载任务起始cid",
        default_value = "0"
    )]
    pub download_start: usize,

    #[structopt(
        long = "enable-filter",
        short = "f",
        help = "开启过滤任务, 默认是下载, 开启后关闭下载专注于数据过滤"
    )]
    pub enable_filter: bool,

    #[structopt(
        long = "filter-name",
        short = "n",
        help = "过滤任务标签",
        default_value = ""
    )]
    pub filter_name: String,

    #[structopt(
        long = "data_path",
        short = "p",
        help = "筛选数据路径",
        default_value = "data"
    )]
    pub data_path: String,

    #[structopt(
        short = "s",
        long = "sql",
        default_value = "192.168.2.25:27017",
        help = "mongodb 服务地址"
    )]
    pub sql: String,
}
