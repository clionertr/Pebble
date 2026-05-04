import os
import re
import glob

# The exact list of RPC endpoints we want to expose (derived from the original tauri::generate_handler)
EXPORTED_COMMANDS = [
    "crate::rpc::health::health_check",
    "crate::rpc::diagnostics::read_app_log",
    "crate::rpc::accounts::add_account",
    "crate::rpc::accounts::get_account_proxy",
    "crate::rpc::accounts::get_account_proxy_setting",
    "crate::rpc::accounts::update_account_proxy",
    "crate::rpc::accounts::update_account_proxy_setting",
    "crate::rpc::accounts::update_account",
    "crate::rpc::accounts::list_accounts",
    "crate::rpc::accounts::delete_account",
    "crate::rpc::accounts::test_imap_connection",
    "crate::rpc::accounts::test_account_connection",
    "crate::rpc::folders::list_folders",
    "crate::rpc::messages::query::list_messages",
    "crate::rpc::messages::query::list_starred_messages",
    "crate::rpc::messages::query::get_message",
    "crate::rpc::messages::query::get_messages_batch",
    "crate::rpc::messages::rendering::get_rendered_html",
    "crate::rpc::messages::rendering::get_message_with_html",
    "crate::rpc::messages::flags::update_message_flags",
    "crate::rpc::messages::rendering::is_trusted_sender",
    "crate::rpc::messages::lifecycle::archive_message",
    "crate::rpc::messages::lifecycle::delete_message",
    "crate::rpc::messages::lifecycle::restore_message",
    "crate::rpc::messages::lifecycle::empty_trash",
    "crate::rpc::messages::lifecycle::move_to_folder",
    "crate::rpc::network::get_global_proxy",
    "crate::rpc::network::update_global_proxy",
    "crate::rpc::search::search_messages",
    "crate::rpc::sync_cmd::start_sync",
    "crate::rpc::sync_cmd::trigger_sync",
    "crate::rpc::sync_cmd::stop_sync",
    "crate::rpc::sync_cmd::set_realtime_preference",
    "crate::rpc::kanban::move_to_kanban",
    "crate::rpc::kanban::list_kanban_cards",
    "crate::rpc::kanban::remove_from_kanban",
    "crate::rpc::kanban::list_kanban_context_notes",
    "crate::rpc::kanban::set_kanban_context_note",
    "crate::rpc::kanban::merge_kanban_context_notes",
    "crate::rpc::labels::get_message_labels",
    "crate::rpc::labels::get_message_labels_batch",
    "crate::rpc::labels::add_message_label",
    "crate::rpc::labels::remove_message_label",
    "crate::rpc::labels::list_labels",
    "crate::rpc::snooze::snooze_message",
    "crate::rpc::snooze::unsnooze_message",
    "crate::rpc::snooze::list_snoozed",
    "crate::rpc::rules::create_rule",
    "crate::rpc::rules::list_rules",
    "crate::rpc::rules::update_rule",
    "crate::rpc::rules::delete_rule",
    "crate::rpc::compose::send_email",
    "crate::rpc::compose::stage_compose_attachment",
    "crate::rpc::trusted_senders::trust_sender",
    "crate::rpc::trusted_senders::list_trusted_senders",
    "crate::rpc::trusted_senders::remove_trusted_sender",
    "crate::rpc::translate::translate_text",
    "crate::rpc::translate::get_translate_config",
    "crate::rpc::translate::save_translate_config",
    "crate::rpc::translate::test_translate_connection",
    "crate::rpc::threads::list_thread_messages",
    "crate::rpc::threads::list_threads",
    "crate::rpc::oauth::complete_oauth_flow",
    "crate::rpc::oauth::get_oauth_account_proxy",
    "crate::rpc::oauth::get_oauth_account_proxy_setting",
    "crate::rpc::oauth::update_oauth_account_proxy",
    "crate::rpc::oauth::update_oauth_account_proxy_setting",
    "crate::rpc::attachments::list_attachments",
    "crate::rpc::attachments::get_attachment_path",
    "crate::rpc::attachments::download_attachment",
    "crate::rpc::batch::batch_archive",
    "crate::rpc::batch::batch_delete",
    "crate::rpc::batch::batch_mark_read",
    "crate::rpc::batch::batch_star",
    "crate::rpc::cloud_sync::test_webdav_connection",
    "crate::rpc::cloud_sync::backup_to_webdav",
    "crate::rpc::cloud_sync::preview_webdav_backup",
    "crate::rpc::cloud_sync::restore_from_webdav",
    "crate::rpc::contacts::search_contacts",
    "crate::rpc::advanced_search::advanced_search",
    "crate::rpc::sync_cmd::reindex_search",
    "crate::rpc::notifications::set_notifications_enabled",
    "crate::rpc::pending_mail_ops::get_pending_mail_ops_summary",
    "crate::rpc::pending_mail_ops::list_pending_mail_ops",
    "crate::rpc::drafts::save_draft",
    "crate::rpc::drafts::delete_draft",
    "crate::rpc::folder_counts::get_folder_unread_counts",
    "crate::rpc::user_data::list_email_templates",
    "crate::rpc::user_data::save_email_template",
    "crate::rpc::user_data::delete_email_template",
    "crate::rpc::user_data::get_email_signature",
    "crate::rpc::user_data::set_email_signature",
]

def parse_rust_files(src_dir):
    functions = []
    
    # Pre-process target function names to filter faster
    target_names = {cmd.split("::")[-1]: cmd for cmd in EXPORTED_COMMANDS}
    
    for filepath in glob.glob(os.path.join(src_dir, "**/*.rs"), recursive=True):
        if "dispatch.rs" in filepath or "mod.rs" in filepath:
            continue
            
        with open(filepath, "r") as f:
            content = f.read()
            
        # regex to match pub fn or pub async fn
        pattern = re.compile(r'pub\s+(async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\((.*?)\)\s*(?:->\s*(.*?))?\s*\{', re.DOTALL)
        
        module_path = filepath.replace(src_dir, "").strip("/").replace(".rs", "").replace("/", "::")
        if module_path.endswith("::mod"):
            module_path = module_path[:-5]
            
        for match in pattern.finditer(content):
            name = match.group(2)
            
            if name not in target_names:
                continue
                
            full_path = target_names[name]
            if not full_path.endswith(f"::{name}"):
                continue
            
            is_async = bool(match.group(1))
            args_str = match.group(3)
            ret_str = match.group(4)
            
            # parse args
            args = []
            arg_parts = []
            current_arg = ""
            bracket_level = 0
            for char in args_str:
                if char == '<' or char == '(' or char == '{' or char == '[':
                    bracket_level += 1
                elif char == '>' or char == ')' or char == '}' or char == ']':
                    bracket_level -= 1
                    
                if char == ',' and bracket_level == 0:
                    arg_parts.append(current_arg)
                    current_arg = ""
                else:
                    current_arg += char
            if current_arg.strip():
                arg_parts.append(current_arg)
            
            for arg_part in arg_parts:
                arg_part = arg_part.strip()
                if not arg_part: continue
                if ":" not in arg_part: continue
                # split at first colon only
                arg_name, arg_type = arg_part.split(":", 1)
                arg_name = arg_name.strip()
                arg_type = arg_type.strip()
                
                # if arg_name is something like 'mut state', clean it up
                arg_name = arg_name.replace("mut ", "").strip()
                
                args.append((arg_name, arg_type))
            
            functions.append({
                "full_path": full_path,
                "name": name,
                "is_async": is_async,
                "args": args,
                "ret": ret_str
            })
            
    return functions

def generate_dispatch(functions, output_file):
    lines = []
    lines.append("use std::sync::Arc;")
    lines.append("use axum::{extract::State, Json, response::IntoResponse};")
    lines.append("use serde_json::{Value, json};")
    lines.append("use crate::state::AppState;")
    lines.append("")
    lines.append("#[derive(serde::Deserialize)]")
    lines.append("pub struct RpcRequest {")
    lines.append("    pub method: String,")
    lines.append("    #[serde(default)]")
    lines.append("    pub params: Value,")
    lines.append("}")
    lines.append("")
    lines.append("pub async fn handle_rpc(")
    lines.append("    State(state): State<Arc<AppState>>,")
    lines.append("    Json(req): Json<RpcRequest>,")
    lines.append(") -> Result<Json<Value>, Json<Value>> {")
    lines.append("    match req.method.as_str() {")
    
    for fn in functions:
        lines.append(f'        "{fn["name"]}" => {{')
        
        args_struct_lines = []
        call_args = []
        
        for arg_name, arg_type in fn["args"]:
            if "State" in arg_type and "AppState" in arg_type:
                # We need to pass axum::extract::State(state.clone()) if the function takes State<Arc<AppState>>
                # The function actually takes axum::extract::State<Arc<AppState>> now!
                call_args.append("axum::extract::State(state.clone())")
            elif "AppHandle" in arg_type or "/* app */" in arg_type:
                # AppHandle needs to be ignored or passed a dummy
                call_args.append("todo!(\"AppHandle not supported\")")
            elif "Window" in arg_type or "/* window */" in arg_type:
                call_args.append("todo!(\"Window not supported\")")
            else:
                # Fully qualify any struct references if needed, but easiest is just to assume they are correctly imported 
                # OR we can just use serde_json::Value and parse directly into the target.
                # Actually, let's fix the imports by using Value and deserializing into the actual type in place? No, struct Args is fine, but we might have missing types.
                # Let's fix missing types by just using serde_json::Value for everything inside Args, then deserializing.
                # Wait, if we use Value, we still need to pass the real type.
                args_struct_lines.append("            #[serde(default)]")
                args_struct_lines.append(f"            pub {arg_name}: serde_json::Value,")
                call_args.append(f"serde_json::from_value(args.{arg_name}).map_err(|e| Json(json!({{ \"error\": format!(\"Invalid arg '{arg_name}': {{}}\", e) }})))?")
                
        if args_struct_lines:
            lines.append("            #[derive(serde::Deserialize)]")
            lines.append("            #[serde(rename_all = \"camelCase\")]")
            lines.append("            struct Args {")
            for line in args_struct_lines:
                lines.append(line)
            lines.append("            }")
            lines.append("            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ \"error\": e.to_string() })))?;")
            
        call_str = f"{fn['full_path']}(" + ", ".join(call_args) + ")"
        if fn["is_async"]:
            call_str += ".await"
            
        lines.append(f"            let res = {call_str};")
        
        if fn["ret"] and "Result" in str(fn["ret"]):
            lines.append("            match res {")
            lines.append("                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),")
            lines.append("                Err(err) => Err(Json(json!({ \"error\": err.to_string() }))),")
            lines.append("            }")
        else:
            lines.append("            Ok(Json(serde_json::to_value(res).unwrap_or(Value::Null)))")
            
        lines.append("        }")
        
    lines.append('        _ => Err(Json(json!({ "error": format!("Unknown method: {}", req.method) }))),')
    lines.append("    }")
    lines.append("}")
    
    with open(output_file, "w") as f:
        f.write("\n".join(lines))

if __name__ == "__main__":
    funcs = parse_rust_files("src-tauri/src/rpc")
    generate_dispatch(funcs, "src-tauri/src/rpc/dispatch.rs")
    print(f"Generated dispatch.rs with {len(funcs)} commands.")
