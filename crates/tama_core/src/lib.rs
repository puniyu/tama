mod error;
pub use error::Error;

pub trait Options {}

/// 运行时抽象 trait
pub trait Runtime: Sized {
    type Options: Options;

    /// 使用给定选项创建运行时实例
    fn new(options: Self::Options) -> Result<Self, Error>;
}
