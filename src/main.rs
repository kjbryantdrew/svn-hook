use clap::{App, SubCommand};
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::process::Command;
use reqwest;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    openai_api_key: String,
    openai_url: String,
    openai_model: String,
    user_language: String,
}

// 获取配置文件路径
fn get_config_file() -> Option<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        let config_dir = home_dir.join(".config").join("commit_crafter");
        
        // 创建目录（如果不存在）
        if !config_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                println!("警告: 创建配置目录失败: {}", e);
                return None;
            }
        }
        
        Some(config_dir.join("config.toml"))
    } else {
        None
    }
}

// 检查配置是否存在
fn check_config() -> bool {
    let config_file = match get_config_file() {
        Some(path) => path,
        None => {
            println!("错误: 无法获取配置目录");
            println!("配置文件应位于:");
            if cfg!(target_os = "windows") {
                println!("  Windows: %APPDATA%\\commit_crafter\\config.toml");
            } else {
                println!("  Unix/macOS: ~/.config/commit_crafter/config.toml");
            }
            return false;
        }
    };

    if !config_file.exists() {
        println!("错误: 未找到配置文件 {}", config_file.display());
        println!("\n请创建配置文件并添加以下内容:");
        println!("```toml");
        println!("openai_api_key = \"your-api-key\"");
        println!("openai_url = \"https://api.openai.com/v1\"");
        println!("openai_model = \"gpt-3.5-turbo\"");
        println!("user_language = \"zh\"");
        println!("```");
        return false;
    }

    // 尝试读取配置
    match fs::read_to_string(&config_file) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(config) => {
                if config.openai_api_key.is_empty() {
                    println!("错误: OpenAI API Key 未设置");
                    return false;
                }
                true
            }
            Err(e) => {
                println!("错误: 解析配置文件失败: {}", e);
                false
            }
        }
        Err(e) => {
            println!("错误: 读取配置文件失败: {}", e);
            false
        }
    }
}

fn check_svn_installed() -> bool {
    match std::process::Command::new("svn").arg("--version").output() {
        Ok(_) => true,
        Err(_) => {
            println!("错误: 未安装 SVN");
            println!("请先安装 SVN:");
            println!("  Ubuntu/Debian: sudo apt install subversion");
            println!("  CentOS/RHEL:  sudo yum install subversion");
            println!("  macOS:        brew install subversion");
            false
        }
    }
}

fn get_svn_diff_for_files(files: &[String]) -> Result<String, String> {
    let mut command = Command::new("svn");
    command.arg("diff");
    
    // 如果指定了文件，添加到命令中
    for file in files {
        command.arg(file);
    }

    let output = command
        .output()
        .map_err(|e| format!("执行 svn diff 失败: {}", e))?;

    if !output.status.success() {
        return Err(format!("svn diff 返回错误: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn commit_with_message_for_files(message: &str, files: &[String]) -> Result<(), String> {
    let mut command = Command::new("svn");
    command.arg("commit")
        .arg("-m")
        .arg(message);
    
    // 如果指定了文件，添加到命令中
    for file in files {
        command.arg(file);
    }

    let output = command
        .output()
        .map_err(|e| format!("执行 svn commit 失败: {}", e))?;

    if !output.status.success() {
        return Err(format!("svn commit 返回错误: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

// 添加一个函数来获取配置
fn get_config() -> Result<Config, String> {
    let config_file = get_config_file().ok_or("无法获取配置目录")?;
    let contents = fs::read_to_string(&config_file)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    toml::from_str(&contents)
        .map_err(|e| format!("解析配置文件失败: {}", e))
}

fn get_system_prompt(language: &str) -> String {
    match language {
        "zh" => format!(
            "你是一个代码审查专家，请用中文生成简洁的提交信息。\n\
             要求：\n\
             1. 只描述修改的主要功能和目的\n\
             2. 不要提及具体的修改内容\n\
             3. 保持信息简短，一般不超过一行\n\
             4. 使用动词开头，描述做了什么"
        ),
        "en" => format!(
            "You are a code review expert. Please generate a concise commit message in English.\n\
             Requirements:\n\
             1. Only describe the main functionality and purpose of the changes\n\
             2. Do not mention specific modification details\n\
             3. Keep the message brief, typically one line\n\
             4. Start with a verb, describing what was done"
        ),
        "ja" => format!(
            "あなたはコードレビューの専門家です。簡潔なコミットメッセージを日本語で生成してください。\n\
             要件：\n\
             1. 変更の主な機能と目的のみを説明する\n\
             2. 具体的な変更内容には触れない\n\
             3. メッセージは簡潔に、通常1行以内\n\
             4. 動詞で始め、何をしたかを説明する"
        ),
        _ => format!(
            "You are a code review expert. Please generate a concise commit message in English.\n\
             Requirements:\n\
             1. Only describe the main functionality and purpose of the changes\n\
             2. Do not mention specific modification details\n\
             3. Keep the message brief, typically one line\n\
             4. Start with a verb, describing what was done"
        ),
    }
}

// 添加AI调用函数
fn generate_commit_message(diff: &str) -> Result<String, String> {
    let config = get_config()?;
    
    // 构建API请求
    let client = reqwest::blocking::Client::new();
    let response = client.post(&format!("{}/v1/chat/completions", config.openai_url))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .json(&serde_json::json!({
            "model": config.openai_model,
            "messages": [
                {
                    "role": "system",
                    "content": get_system_prompt(&config.user_language)
                },
                {
                    "role": "user",
                    "content": format!("请根据以下代码变更生成提交信息：\n\n{}", diff)
                }
            ],
            "temperature": 0.7
        }))
        .send()
        .map_err(|e| format!("API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API返回错误: {}", response.text().unwrap_or_default()));
    }

    let response_data: serde_json::Value = response.json()
        .map_err(|e| format!("解析API响应失败: {}", e))?;

    Ok(response_data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("无法获取生成的提交信息")
        .to_string())
}

fn handle_commit(files: Vec<String>) {
    // 首先检查 SVN 是否安装
    if !check_svn_installed() {
        return;
    }

    // 检查配置
    if !check_config() {
        return;
    }

    // 1. 获取变更
    println!("正在获取变更信息...");
    let diff = match get_svn_diff_for_files(&files) {
        Ok(diff) => diff,
        Err(e) => {
            println!("错误: {}", e);
            return;
        }
    };

    if diff.is_empty() {
        println!("没有检测到任何变更");
        return;
    }

    let mut extra_prompt: Option<String> = None;

    loop {
        // 2. 调用AI生成提交信息
        println!("正在生成提交信息...");
        let config = match get_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                println!("获取配置失败: {}", e);
                return;
            }
        };
        
        let commit_message = match if let Some(ref prompt) = extra_prompt {
            generate_commit_message_with_prompt(&diff, prompt)
        } else {
            generate_commit_message(&diff)
        } {
            Ok(message) => message,
            Err(e) => {
                println!("生成提交信息失败: {}", e);
                return;
            }
        };

        // 3. 显示提交信息并等待确认
        println!("\n生成的提交信息 (使用模型: {}):", config.openai_model);
        println!("{}", "-".repeat(50));
        println!("{}", commit_message);
        println!("{}", "-".repeat(50));
        println!("\n选项:");
        println!("1. 使用此提交信息并自动提交 [y]");
        println!("2. 显示提交命令 [s]");
        println!("3. 重新生成 [r]");
        println!("4. 退出 [n]");
        println!("请选择 [Y/s/r/n]:");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        match input.trim().to_lowercase().as_str() {
            "y" | "" => {
                println!("\n正在提交...");
                match commit_with_message_for_files(&commit_message, &files) {
                    Ok(_) => {
                        println!("✅ 提交成功!");
                        break;
                    }
                    Err(e) => {
                        println!("提交失败: {}", e);
                        println!("\n您可以手动执行以下命令:");
                        print!("svn commit -m \"{}\"", commit_message);
                        for file in &files {
                            print!(" {}", file);
                        }
                        println!();
                        break;
                    }
                }
            },
            "s" => {
                println!("\n请使用以下命令提交:");
                print!("svn commit -m \"{}\"", commit_message);
                for file in &files {
                    print!(" {}", file);
                }
                println!();
                break;
            },
            "r" => {
                println!("\n请输入额外的提示信息（可选，直接回车跳过）:");
                let mut prompt = String::new();
                std::io::stdin().read_line(&mut prompt).unwrap();
                if !prompt.trim().is_empty() {
                    extra_prompt = Some(prompt.trim().to_string());
                } else {
                    extra_prompt = None;
                }
                continue;
            },
            "n" => {
                println!("已退出");
                break;
            },
            _ => {
                println!("无效的选择，请重新选择");
                continue;
            }
        }
    }
}

// 添加一个新函数，支持额外的提示信息
fn generate_commit_message_with_prompt(diff: &str, extra_prompt: &str) -> Result<String, String> {
    let config = get_config()?;
    
    // 构建API请求
    let client = reqwest::blocking::Client::new();
    let response = client.post(&format!("{}/v1/chat/completions", config.openai_url))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .json(&serde_json::json!({
            "model": config.openai_model,
            "messages": [
                {
                    "role": "system",
                    "content": get_system_prompt(&config.user_language)
                },
                {
                    "role": "user",
                    "content": format!("请根据以下代码变更生成提交信息：\n\n{}\n\n额外要求：{}", diff, extra_prompt)
                }
            ],
            "temperature": 0.7
        }))
        .send()
        .map_err(|e| format!("API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API返回错误: {}", response.text().unwrap_or_default()));
    }

    let response_data: serde_json::Value = response.json()
        .map_err(|e| format!("解析API响应失败: {}", e))?;

    Ok(response_data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("无法获取生成的提交信息")
        .to_string())
}

fn main() {
    let matches = App::new("svn-hook")
        .version("0.1.0")
        .about("SVN 提交信息生成工具")
        .subcommand(SubCommand::with_name("commit")
            .about("生成提交信息")
            .arg(clap::Arg::with_name("files")
                .help("要提交的文件或目录路径")
                .multiple(true)
                .index(1)))
        .get_matches();

    match matches.subcommand() {
        ("commit", Some(sub_matches)) => {
            let files: Vec<String> = sub_matches.values_of("files")
                .map(|values| values.map(String::from).collect())
                .unwrap_or_else(Vec::new);
            handle_commit(files)
        },
        _ => println!("使用 --help 查看帮助信息"),
    }
} 