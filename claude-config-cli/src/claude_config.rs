use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub struct ClaudeConfigManager {
    directory_path: String,
}

impl ClaudeConfigManager {
    pub fn new(directory_path: String) -> Self {
        Self { directory_path }
    }

    fn get_claude_dir(&self) -> String {
        format!("{}/.claude", self.directory_path)
    }

    fn get_settings_file(&self) -> String {
        format!("{}/settings.local.json", self.get_claude_dir())
    }

    fn get_alternative_settings_files(&self) -> Vec<String> {
        vec![
            format!("{}/settings.json", self.get_claude_dir()),
            format!("{}/claude_config.json", self.get_claude_dir()),
            format!("{}/.claude_config", self.directory_path),
            format!("{}/CLAUDE.md", self.directory_path),
        ]
    }

    fn ensure_claude_dir(&self) -> Result<()> {
        let claude_dir = self.get_claude_dir();
        if !Path::new(&claude_dir).exists() {
            fs::create_dir_all(&claude_dir)?;
        }
        Ok(())
    }

    fn read_settings(&self) -> Result<Value> {
        let settings_file = self.get_settings_file();

        if Path::new(&settings_file).exists() {
            let content = fs::read_to_string(&settings_file)?;
            let settings: Value = serde_json::from_str(&content)?;
            return Ok(settings);
        }

        // 检查其他可能的配置文件
        for alt_file in self.get_alternative_settings_files() {
            if Path::new(&alt_file).exists() {
                // 如果是 CLAUDE.md 文件，需要特殊处理
                if alt_file.ends_with("CLAUDE.md") {
                    return self.parse_claude_md(&alt_file);
                }

                let content = fs::read_to_string(&alt_file)?;
                if let Ok(settings) = serde_json::from_str::<Value>(&content) {
                    return Ok(settings);
                }
            }
        }

        Ok(json!({}))
    }

    fn parse_claude_md(&self, file_path: &str) -> Result<Value> {
        let content = fs::read_to_string(file_path)?;

        // 简单解析CLAUDE.md中的环境变量
        let mut env_config = json!({});

        for line in content.lines() {
            if line.trim().starts_with("ANTHROPIC_API_KEY=") {
                let value = line.split('=').nth(1).unwrap_or("").trim();
                env_config["ANTHROPIC_API_KEY"] = json!(value);
            } else if line.trim().starts_with("ANTHROPIC_BASE_URL=") {
                let value = line.split('=').nth(1).unwrap_or("").trim();
                env_config["ANTHROPIC_BASE_URL"] = json!(value);
            } else if line.trim().starts_with("CLAUDE_API_KEY=") {
                let value = line.split('=').nth(1).unwrap_or("").trim();
                env_config["CLAUDE_API_KEY"] = json!(value);
            }
        }

        if env_config.as_object().unwrap().is_empty() {
            return Ok(json!({}));
        }

        Ok(json!({ "env": env_config }))
    }

    fn write_settings(&self, settings: &Value) -> Result<()> {
        self.ensure_claude_dir()?;
        let settings_file = self.get_settings_file();
        let content = serde_json::to_string_pretty(settings)?;
        fs::write(&settings_file, content)?;
        Ok(())
    }

    pub fn update_env_config_with_options(
        &self,
        token: String,
        base_url: String,
        is_sandbox: bool,
    ) -> Result<bool> {
        let mut settings = self.read_settings()?;

        if !settings.is_object() {
            settings = json!({});
        }

        let mut env_config = json!({
            "ANTHROPIC_API_KEY": token,
            "ANTHROPIC_AUTH_TOKEN": token,
            "ANTHROPIC_BASE_URL": base_url,
        });

        // 添加可选的环境变量
        if is_sandbox {
            env_config["IS_SANDBOX"] = json!("1");
        }

        settings["env"] = env_config;

        self.write_settings(&settings)?;

        // 复制 CLAUDE.local.md 文件
        self.copy_claude_local_md()?;

        Ok(true)
    }

    #[allow(dead_code)]
    pub fn clear_env_config(&self) -> Result<bool> {
        let mut settings = self.read_settings()?;

        if let Some(env) = settings.get_mut("env") {
            if let Some(obj) = env.as_object_mut() {
                obj.remove("ANTHROPIC_API_KEY");
                obj.remove("ANTHROPIC_AUTH_TOKEN");
                obj.remove("ANTHROPIC_BASE_URL");

                if obj.is_empty() {
                    settings.as_object_mut().unwrap().remove("env");
                }
            }
        }

        self.write_settings(&settings)?;
        Ok(true)
    }

    fn copy_claude_local_md(&self) -> Result<()> {
        // 使用 include_str! 在编译时嵌入 CLAUDE.local.md 内容
        const CLAUDE_LOCAL_MD_CONTENT: &str = include_str!("../resources/config/CLAUDE.local.md");

        // 目标文件路径
        let target_file = Path::new(&self.directory_path).join("CLAUDE.local.md");

        // 写入文件
        fs::write(&target_file, CLAUDE_LOCAL_MD_CONTENT)?;

        tracing::info!("成功写入 CLAUDE.local.md 到 {}", target_file.display());

        Ok(())
    }
}
