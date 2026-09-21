//! AZ-900 刷题台（Tauri 桌面版）。
//!
//! 这一层很薄：打开数据库、注册命令、启动窗口。业务逻辑全在 `az900-core`。
mod commands;
mod state;

use state::AppState;
use tauri::Manager;

/// 题库 JSON 在编译时嵌进二进制，发布版不需要外部文件。
const BANK_FILES: &[(&str, &str)] = &[
    (
        "domain1_cloud_concepts.json",
        include_str!("../../../data/questions/domain1_cloud_concepts.json"),
    ),
    (
        "domain2a_architecture_compute_network.json",
        include_str!("../../../data/questions/domain2a_architecture_compute_network.json"),
    ),
    (
        "domain2b_storage_identity_security.json",
        include_str!("../../../data/questions/domain2b_storage_identity_security.json"),
    ),
    (
        "domain3_management_governance.json",
        include_str!("../../../data/questions/domain3_management_governance.json"),
    ),
];

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 数据库放在系统的应用数据目录，卸载重装不丢成绩
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db_path = dir.join("az900.db");

            let state = AppState::init(db_path, BANK_FILES)
                .map_err(|e| format!("初始化失败：{e}"))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bank_size,
            commands::get_topics,
            commands::get_question,
            commands::start_exam,
            commands::start_practice,
            commands::start_review,
            commands::submit_answer,
            commands::finish_session,
            commands::get_wrongbook,
            commands::clear_wrongbook,
            commands::get_dashboard,
            commands::get_db_path,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}
