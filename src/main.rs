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

// 检查 Git 是否安装
fn check_git_installed() -> bool {
    match std::process::Command::new("git").arg("--version").output() {
        Ok(_) => true,
        Err(_) => {
            // Git 是可选的，所以只打印一条信息
            println!("信息: 未检测到 Git，将跳过 Git 相关操作。");
            false
        }
    }
}

// 检查当前目录是否为 Git 仓库
fn is_git_repository() -> bool {
    if !check_git_installed() {
        return false;
    }
    match Command::new("git").args(&["rev-parse", "--is-inside-work-tree"]).output() {
        Ok(output) => {
            output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "true"
        }
        Err(_) => false,
    }
}

// 向 Git 提交
fn commit_to_git(message: &str, files: &[String]) -> Result<(), String> {
    println!("\n检测到 Git 仓库。");

    // 构造将要执行的命令字符串用于显示
    let mut git_add_cmd_str = "git add".to_string();
    if files.is_empty() {
        git_add_cmd_str.push_str(" .");
    } else {
        for file in files {
            git_add_cmd_str.push(' ');
            git_add_cmd_str.push_str(file);
        }
    }
    let git_commit_cmd_str = format!("git commit -m \"{}\"", message);

    println!("将执行以下命令:");
    println!("  {}", git_add_cmd_str);
    println!("  {}", git_commit_cmd_str);
    println!("\n是否确认执行 Git 提交? [Y/n]:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    match input.trim().to_lowercase().as_str() {
        "y" | "" => {
            println!("正在执行 Git 提交...");
            // 根据是否提供了文件列表来决定是添加指定文件还是所有变更
            let mut add_command = Command::new("git");
            add_command.arg("add");
            if files.is_empty() {
                // 如果没有指定文件，则添加所有变更
                add_command.arg(".");
            } else {
                // 否则只添加指定的文件
                for file in files {
                    add_command.arg(file);
                }
            }

            let add_output = add_command
                .output()
                .map_err(|e| format!("执行 git add 失败: {}", e))?;

            if !add_output.status.success() {
                return Err(format!(
                    "git add 返回错误: {}",
                    String::from_utf8_lossy(&add_output.stderr)
                ));
            }

            // git commit -m "message"
            let commit_output = Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(message)
                .output()
                .map_err(|e| format!("执行 git commit 失败: {}", e))?;

            if !commit_output.status.success() {
                let stderr = String::from_utf8_lossy(&commit_output.stderr);
                if stderr.contains("nothing to commit") || stderr.contains("无文件要提交") {
                    println!("Git 仓库没有变更，无需提交。");
                    return Ok(());
                }
                return Err(format!(
                    "git commit 返回错误: {}",
                    stderr
                ));
            }

            println!("✅ Git 提交成功!");
            Ok(())
        }
        "n" => {
            println!("已取消 Git 提交。");
            Ok(())
        }
        _ => {
            println!("无效的选择，已取消 Git 提交。");
            Ok(())
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
    command.arg("commit").arg("-m").arg(message);
    for file in files {
        command.arg(file);
    }

    let output = command
        .output()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        // 检查是否需要输入密码
        if error.contains("无法取得密码") || error.contains("Unable to connect") {
            println!("需要输入密码，正在切换到交互模式...");
            
            // 重新执行命令，这次使用spawn让SVN可以直接与终端交互
            let mut interactive_command = Command::new("svn");
            interactive_command.arg("commit").arg("-m").arg(message);
            for file in files {
                interactive_command.arg(file);
            }
            
            let mut child = interactive_command
                .spawn()
                .map_err(|e| format!("执行命令失败: {}", e))?;
            
            // 等待命令完成
            let status = child.wait()
                .map_err(|e| format!("等待命令完成失败: {}", e))?;

            if !status.success() {
                return Err("提交失败，请检查密码是否正确".to_string());
            }
            return Ok(());
        }
        return Err(error.to_string());
    }

    Ok(())
}

// 获取配置
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
            "生成精简的SVN提交信息：\n\
             1. 控制在15字以内\n\
             2. 只提取最核心动作和目的\n\
             3. 忽略具体文件名和细节\n\
             4. 不列举具体项目\n\
             5. 不使用标点符号\n\
             6. 严禁输出任何推理过程或解释\n\
             7. 只输出提交信息本身，不要有其他任何内容"
        ),
        "en" => format!(
            "Generate minimal SVN commit message:\n\
             1. Maximum 8 words\n\
             2. Extract only core action and purpose\n\
             3. Ignore specific filenames and details\n\
             4. No listing of items\n\
             5. No punctuation\n\
             6. Strictly forbidden to output any reasoning process\n\
             7. Output only the commit message itself with no other content"
        ),
        "ja" => format!(
            "簡潔なSVNコミットメッセージ：\n\
             1. 15字以内\n\
             2. 核心動作と目的のみ\n\
             3. ファイル名や詳細は無視\n\
             4. 項目列挙禁止\n\
             5. 句読点使用禁止\n\
             6. 推論過程の出力は厳禁\n\
             7. コミットメッセージのみを出力し他の内容は含めない"
        ),
        _ => format!(
            "Generate minimal SVN commit message:\n\
             1. Maximum 8 words\n\
             2. Extract only core action and purpose\n\
             3. Ignore specific filenames and details\n\
             4. No listing of items\n\
             5. No punctuation\n\
             6. Strictly forbidden to output any reasoning process\n\
             7. Output only the commit message itself with no other content"
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
                        println!("✅ SVN 提交成功!");

                        // 如果是Git仓库，也提交到Git
                        if is_git_repository() {
                            if let Err(e) = commit_to_git(&commit_message, &files) {
                                println!("警告: Git 提交失败: {}", e);
                            }
                        }
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
            handle_commit(files);
        },
        _ => println!("使用 --help 查看帮助信息"),
    }
}