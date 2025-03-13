use crate::{
    app::AppResult,
    app_err,
    plugin::{helpers::add_helper_globals, Plugin, PluginKind},
    util::toml::Config,
};
use mlua::{Function, Lua, LuaSerdeExt};
use std::{collections::HashMap, sync::Arc};

use super::eval::PluginEvalOutput;

pub fn report(
    plugin: &Plugin,
    toml: &Arc<Config>,
    output: &PluginEvalOutput,
    plugin_id: &str,
) -> AppResult<HashMap<String, String>> {
    if plugin.kind != PluginKind::Report {
        return Err(app_err!("{} is not a reporting plugin.", plugin.details.id));
    }

    let lua = Lua::new();
    add_helper_globals(&lua)?;

    let plugin_config = plugin.get_config_lua(&lua, toml);

    let report: Function = {
        let contents = std::fs::read_to_string(plugin.path.join("run.lua"))
            .expect("Error reading plugin code");

        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Run").unwrap())
    }?;

    let report_state = lua.create_table()?;
    report_state.set("config", plugin_config)?;
    let output_lua = lua.to_value(&output)?;
    report_state.set("output", output_lua)?;
    report_state.set("plugin_id", plugin_id)?;

    let report_results = report.call::<mlua::Value>(report_state)?;

    let report_results: HashMap<String, String> = lua.from_value(report_results)?;

    Ok(report_results)
}
