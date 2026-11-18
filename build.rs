fn main() {
    let config = slint_build::CompilerConfiguration::new().with_style("fluent-dark".into());
    slint_build::compile_with_config("ui/app.slint", config).expect("не удалось собрать Slint UI");

    // В Windows-сборку встраиваются иконка и сведения о файле (версия, описание).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=assets/icon.ico");
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        resource.set("ProductName", "Stat Forge");
        resource.set(
            "FileDescription",
            "Stat Forge — генератор характеристик D&D",
        );
        resource
            .compile()
            .expect("не удалось встроить ресурсы Windows");
    }
}
