use tama_core::{Error, Options, Runtime};

/// 基于 wasmtime Config 的运行时选项
///
/// 封装了 `wasmtime::Config`。
/// 通过 `Deref`/`DerefMut` 可直接访问底层 wasmtime Config 的所有配置项。
#[derive(Clone, Default)]
pub struct WasmRuntimeOptions {
    config: wasmtime::Config,
}

impl WasmRuntimeOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::ops::Deref for WasmRuntimeOptions {
    type Target = wasmtime::Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl std::ops::DerefMut for WasmRuntimeOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl Options for WasmRuntimeOptions {}

/// 基于 wasmtime 的 WebAssembly 运行时
///
/// 持有 `Engine` 和 `Linker`，提供模块加载、实例化和函数调用能力。
/// WASI 等宿主能力通过 `linker_mut()` 由外部注入。
pub struct WasmRuntime {
    engine: wasmtime::Engine,
    linker: wasmtime::Linker<()>,
}

impl Runtime for WasmRuntime {
    type Options = WasmRuntimeOptions;

    fn new(options: Self::Options) -> Result<Self, Error> {
        let engine = wasmtime::Engine::new(&options.config).map_err(|e| {
            Error::Runtime(Box::<dyn std::error::Error + Send + Sync>::from(
                e.to_string(),
            ))
        })?;

        let linker = wasmtime::Linker::new(&engine);

        Ok(Self { engine, linker })
    }
}

impl WasmRuntime {
    /// 获取底层 Engine 的引用
    pub fn engine(&self) -> &wasmtime::Engine {
        &self.engine
    }

    /// 获取 Linker 的引用
    pub fn linker(&self) -> &wasmtime::Linker<()> {
        &self.linker
    }

    /// 获取 Linker 的可变引用，用于注入宿主函数（如 WASI）
    ///
    /// # 示例
    /// ```ignore
    /// // 注入 WASI（由外部 crate 负责，如 wasmtime-wasi）
    /// wasmtime_wasi::p1::add_to_linker_sync(runtime.linker_mut(), |s| s)?;
    /// ```
    pub fn linker_mut(&mut self) -> &mut wasmtime::Linker<()> {
        &mut self.linker
    }

    /// 编译 Wasm 模块
    pub fn compile(&self, wasm: &[u8]) -> Result<wasmtime::Module, Error> {
        wasmtime::Module::new(&self.engine, wasm).map_err(|e| {
            Error::Runtime(Box::<dyn std::error::Error + Send + Sync>::from(
                e.to_string(),
            ))
        })
    }

    /// 从文件路径编译 Wasm 模块
    pub fn compile_file(&self, path: impl AsRef<std::path::Path>) -> Result<wasmtime::Module, Error> {
        wasmtime::Module::from_file(&self.engine, path).map_err(|e| {
            Error::Runtime(Box::<dyn std::error::Error + Send + Sync>::from(
                e.to_string(),
            ))
        })
    }

    /// 使用 Linker 实例化模块
    pub fn instantiate(&self, module: &wasmtime::Module) -> Result<wasmtime::Instance, Error> {
        let mut store = wasmtime::Store::new(&self.engine, ());
        self.linker.instantiate(&mut store, module).map_err(|e| {
            Error::Runtime(Box::<dyn std::error::Error + Send + Sync>::from(
                e.to_string(),
            ))
        })
    }

    /// 调用实例的导出函数（类型安全）
    ///
    /// # 示例
    /// ```ignore
    /// let result: i32 = runtime.call(&instance, "add", (1i32, 2i32))?;
    /// ```
    pub fn call<Params, Results>(
        &self,
        instance: &wasmtime::Instance,
        name: &str,
        params: Params,
    ) -> Result<Results, Error>
    where
        Params: wasmtime::WasmParams,
        Results: wasmtime::WasmResults,
    {
        let mut store = wasmtime::Store::new(&self.engine, ());
        let func = instance.get_typed_func(&mut store, name).map_err(|e| {
            Error::Runtime(Box::<dyn std::error::Error + Send + Sync>::from(
                e.to_string(),
            ))
        })?;
        func.call(&mut store, params).map_err(|e| {
            Error::Runtime(Box::<dyn std::error::Error + Send + Sync>::from(
                e.to_string(),
            ))
        })
    }
}
