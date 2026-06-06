#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 运行时内部错误
    #[error("runtime error: {0}")]
    Runtime(#[from] Box<dyn std::error::Error + Send + Sync>),
    /// 无效的选项配置
    #[error("invalid options: {0}")]
    InvalidOptions(String),
}