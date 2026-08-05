use stayhydated_xtask::trunk::{TrunkDemoBuildConfig, TrunkDemoPageConfig};

pub fn run() -> anyhow::Result<()> {
    let workspace_root = stayhydated_xtask::workspace_root_from_xtask_manifest()?;
    stayhydated_xtask::trunk::build(
        &TrunkDemoBuildConfig::builder()
            .workspace_root(workspace_root)
            .example_dir("examples/some-lib-tables")
            .output_dir("web/public/gpui-demo")
            .example_name("demo")
            .required_marker("gpui-table-some-lib-tables")
            .toolchain("nightly")
            .generated_page(
                TrunkDemoPageConfig::builder()
                    .title("some-lib-tables Storybook demo")
                    .demo_name("some-lib-tables Storybook")
                    .build(),
            )
            .build(),
    )
}
