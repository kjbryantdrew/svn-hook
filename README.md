[中文版 (Chinese)](README_zh-CN.md)

# SVN AI Commit Hook - Intelligent SVN Commit Message Assistant

`SVN AI Commit Hook` is a powerful Subversion (SVN) pre-commit hook script that leverages the capabilities of Artificial Intelligence (AI) to automatically generate standardized, clear, and informative commit messages based on your staged changes. Say goodbye to the tedious task of manually writing commit messages and let AI help you improve version control efficiency and quality.

## ✨ Core Features

- **🤖 AI-Driven Commit Message Generation**: Automatically triggers when you execute the `svn commit` command (if no commit message is provided via `-m` or `-F` arguments), analyzes the differences (diff) of the current commit, and calls an AI model (like the GPT series) to generate high-quality commit messages.
- **📝 Standardization and Consistency**: Ensures a unified style for team commit messages, adhering to preset formats and language requirements (e.g., Chinese or English, and specific commit type prefixes).
- **⏱️ Save Time, Enhance Efficiency**: Significantly reduces the time developers spend thinking about and writing commit messages, allowing them to focus more on coding.
- **💡 Context-Aware**: The AI can understand the intent of code changes, generating more descriptive commit messages than generic templates.
- **⚙️ Flexible Configuration**: Through the shared configuration file `~/.config/commit_crafter/config.toml`, you can easily set API keys, AI models, and the desired commit message language.
- **💬 Interactive Confirmation**: Before automatically committing, it displays the AI-generated commit message and allows you to **confirm usage**, **manually edit**, or **cancel the current commit**, ensuring you have full control over the final commit message.
- **🤝 Git Integration (Optional)**: If the SVN project is also a Git repository, the tool will offer to create a corresponding Git commit with the same AI-generated message after a successful SVN commit.

## 🤝 Git Integration

If your SVN working copy is also a Git repository, `svn-hook` provides an additional feature to keep your repositories in sync.

### How It Works

1.  **Automatic Detection**: The tool automatically checks if the current directory is a Git repository.
2.  **SVN Commit First**: The standard SVN commit process proceeds as usual.
3.  **Confirmation for Git Commit**: After a successful SVN commit, if a Git repository is detected, the tool will prompt you for a secondary Git commit.
    - It will display the exact `git add` and `git commit` commands that will be executed.
    - The files added to the Git stage will be the same as those you specified for the SVN commit. If you didn't specify any files (committing all changes), it will use `git add .`.
4.  **User Control**: The Git commit is **only** executed if you confirm the prompt. You can safely decline to skip the Git commit for that instance.

This feature ensures that for hybrid projects, your commit history can be consistently maintained across both version control systems with minimal extra effort.

## 📋 Prerequisites and Configuration

This hook script is actually a Rust-compiled executable program that handles interaction with the AI service and user prompts.

1.  **Rust Executable Program (`svn-hook`)**:
    Ensure you have compiled and installed the `svn-hook` program using the `install.sh` script provided in the project. This script typically installs the executable to `~/.cargo/bin/`. Make sure this directory is in your system's `PATH` environment variable.

2.  **Create and Configure `config.toml`**:
    This tool shares the configuration file located at `~/.config/commit_crafter/config.toml` with `File Organizer`.
    If you haven't configured it yet, please do the following:
    ```bash
    # Create directory (if it doesn't exist)
    mkdir -p ~/.config/commit_crafter
    
    # Create configuration file (if it doesn't exist)
    touch ~/.config/commit_crafter/config.toml
    ```
    Fill in the `config.toml` file with the following content:
    ```toml
    # Your OpenAI API Key
    openai_api_key = "sk-your-key-here"

    # Base URL for the OpenAI API (modify if using a proxy)
    openai_url = "https://api.openai.com"

    # AI model for generating commit messages
    openai_model = "gpt-4-turbo" # or your preferred model

    # (Optional) Specify the language for AI to use when generating commit messages
    user_language = "Chinese" # or "English"
    ```

## 🔧 SVN Hook Configuration

To enable this AI commit assistant, you need to configure a `pre-commit` hook in your SVN repository. This hook script will call our compiled `svn-hook` Rust program.

1.  **Locate Hook Directory**:
    For server-side hooks (recommended), navigate to the `hooks` directory of your SVN repository on the server.
    For client-side hooks (may not be supported or recommended by some SVN versions/configurations), it might be in the `.svn/hooks/` directory of the working copy (but this is generally for local validation, not centralized control).

2.  **Create `pre-commit` Script**:
    In the hook directory, create an executable script named `pre-commit` (e.g., on Linux/macOS). On Windows, it might be `pre-commit.bat`.

3.  **Example `pre-commit` Script Content (Linux/macOS)**:
    ```bash
    #!/bin/bash

    REPOS="$1"  # Repository path (passed by SVN)
    TXN="$2"    # Transaction name (passed by SVN)

    # Path to the svn-hook executable (if in PATH, just 'svn-hook')
    SVN_HOOK_EXEC="svn-hook"

    # Check if the user provided a commit message via -m or -F
    # svnlook log "$REPOS" -t "$TXN" would return that message.
    # The svn-hook program itself handles this logic: if a message is provided, it exits and allows the commit.

    # Create a temporary file to store the output of svn-hook (the final commit message)
    TMP_MSG_FILE=$(mktemp)

    # Call the svn-hook Rust program.
    # It will interact with the user, and if confirmed, print the final commit message to stdout.
    # Errors will be printed to stderr.
    if $SVN_HOOK_EXEC --repo-path "$REPOS" --txn "$TXN" > "$TMP_MSG_FILE"; then
        # svn-hook exited successfully (user confirmed the commit message)
        # Set the svn:log property using the content of the temporary file.
        if [ -s "$TMP_MSG_FILE" ]; then # Ensure the temp file has content
            svnlook propset --txn "$TXN" "$REPOS" svn:log -F "$TMP_MSG_FILE"
            rm "$TMP_MSG_FILE"
            exit 0 # Allow commit
        else
            # Should not happen: successful exit but no commit message
            echo "Error: svn-hook executed successfully but provided no commit message." >&2
            rm "$TMP_MSG_FILE"
            exit 1 # Block commit
        fi
    else
        # svn-hook exited with an error (user cancelled, AI call failed, etc.)
        # The Rust program should have printed an error message to stderr, which the SVN client will display.
        rm "$TMP_MSG_FILE"
        exit 1 # Block commit
    fi
    ```
    **Important Notes**:
    *   Ensure the `pre-commit` script has execute permissions (`chmod +x pre-commit`).
    *   The `SVN_HOOK_EXEC` variable should point to your compiled and installed `svn-hook` Rust program. If it's in the system's `PATH`, `svn-hook` is sufficient.
    *   The core logic of this script is: call the `svn-hook` program, capture its standard output (the user-confirmed commit message), and use this message to set the `svn:log` property for the current SVN transaction.

## 💡 Usage Flow

1.  Make code changes in your SVN working copy.
2.  Execute `svn add ...` (if adding new files) and then `svn commit`.
3.  When you execute `svn commit`:
    *   If you **do not** provide a commit message via `-m "message"` or `-F file`, the `pre-commit` hook will trigger the `svn-hook` program.
    *   If you **have** provided a commit message, the `svn-hook` program will detect this and allow the commit directly, skipping the AI generation step.
4.  If AI is triggered: The AI will analyze your changes and generate a suggested commit message.
5.  You will see the suggested commit message in your terminal and can choose:
    - **(y)es**: Commit directly using the AI-generated message.
    - **(e)dit**: Modify the AI-generated message in your default text editor, then save and close the editor to commit.
    - **(n)o**: Cancel the current commit.
6.  **Git Commit (Optional)**:
    *   After a successful SVN commit, if the tool detects that the current directory is also a Git repository, it will display the `git` commands to be executed and ask if you want to proceed with a Git commit.
    *   You can choose **(y)es** to confirm and complete the Git commit, or **(n)o** to skip it.

---
*This tool was developed with the assistance of Cascade (a world-class AI coding assistant).*

一个基于 Rust 实现的 SVN 提交信息生成工具，通过调用大模型 API 自动生成规范的提交说明。

## 项目描述

在日常开发中，编写规范的 SVN 提交信息是一个重要但容易被忽视的任务。本工具旨在通过 AI 技术自动分析代码变更，生成清晰、规范的提交信息，提高代码管理的质量和效率。

本项目的思路和配置设计来源于 [commit_crafter](https://github.com/yzzting/commit_crafter)，这是一个优秀的 Git 提交信息生成工具。我们将其核心理念应用到 SVN 环境中，并复用了其配置文件设计，以保持一致的用户体验。

## 技术特点

- **Rust 实现**：使用 Rust 语言开发，保证了工具的性能和安全性
- **AI 驱动**：集成大模型 API，智能分析代码变更
- **配置复用**：复用 commit_crafter 的配置文件，降低配置成本
- **交互友好**：提供清晰的命令行交互界面
- **灵活可选**：支持自动提交或手动执行
- **可定制化**：支持自定义提示词，优化生成结果

## 实现思路

1. **代码分析**
   - 使用 SVN 命令获取待提交的变更内容
   - 分析变更类型（新增、修改、删除）
   - 提取关键信息用于生成提交说明

2. **AI 生成**
   - 构建结构化的提示信息
   - 调用大模型 API 生成提交说明
   - 确保生成内容简洁规范

3. **交互处理**
   - 提供多种操作选项
   - 支持重新生成和自定义提示
   - 实现自动提交功能

## 实现方式

### 核心组件

1. **配置管理**
   ```rust
   struct Config {
       openai_api_key: String,
       openai_url: String,
       openai_model: String,
       user_language: String,
   }
   ```

2. **SVN 交互**
   - 使用 `svn diff` 获取变更
   - 使用 `svn commit` 执行提交

3. **AI 调用**
   - 使用 OpenAI API
   - 支持自定义模型和接口

### 工作流程

1. 检查环境（SVN、配置文件）
2. 获取代码变更
3. 调用 AI 生成提交信息
4. 用户确认或重新生成
5. 执行提交或显示命令

## 安装部署

### 环境要求

- Rust 1.56.0+
- SVN 命令行工具
- commit_crafter 配置

### 安装步骤

1. 安装 Rust（如果未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. 编译安装：

   方式一：使用安装脚本（推荐）
   ```bash
   git clone <repository_url>
   cd svn_hook
   chmod +x install.sh
   ./install.sh
   ```

   方式二：手动安装
   ```bash
   git clone <repository_url>
   cd svn_hook
   cargo install --path .
   rm -rf target/  # 清理编译生成的临时文件
   ```

### 配置文件

#### 文件路径
配置文件位置（与 commit_crafter 保持一致）：

- Unix/macOS: `~/.config/commit_crafter/config.toml`
- Windows: `%APPDATA%\commit_crafter\config.toml`

首次使用时，需要创建配置文件并添加以下内容：

```toml
openai_api_key = "your-api-key"
openai_url = "https://api.openai.com/v1"
openai_model = "gpt-3.5-turbo"
user_language = "zh"
```

#### 配置项说明

```toml
# OpenAI API密钥
openai_api_key = "your-api-key"

# API接口地址
# 默认使用 OpenAI 官方接口
openai_url = "https://api.openai.com/v1"
# 如果使用代理，可以修改为代理地址
# openai_url = "https://your-proxy-url/v1"

# 使用的模型
# 可选值：
# - gpt-4（如果有访问权限）
# - gpt-3.5-turbo（默认）
# - 其他支持的模型
openai_model = "gpt-3.5-turbo"

# 生成提交信息的语言
# 可选值：
# - zh：中文（默认）
# - en：英文
# - ja：日文
# 等其他语言代码
user_language = "zh"
```

#### 配置示例

1. 基础配置（使用默认值）：
```toml
openai_api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
openai_url = "https://api.openai.com/v1"
openai_model = "gpt-3.5-turbo"
user_language = "zh"
```

2. 使用代理服务器：
```toml
openai_api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
openai_url = "https://your-proxy-server.com/v1"
openai_model = "gpt-3.5-turbo"
user_language = "zh"
```

3. 使用 GPT-4 模型和英文输出：
```toml
openai_api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
openai_url = "https://api.openai.com/v1"
openai_model = "gpt-4"
user_language = "en"
```

#### 配置注意事项

1. API 密钥安全：
   - 不要将包含真实 API 密钥的配置文件提交到版本控制系统
   - 建议设置适当的文件权限（如：`chmod 600 config.toml`）

2. 配置文件格式：
   - 必须使用 TOML 格式
   - 所有字段都是必需的
   - 字符串值需要使用双引号

3. 故障排除：
   - 如果工具报错找不到配置文件，检查文件路径是否正确
   - 如果 API 调用失败，检查 API 密钥和 URL 是否正确
   - 如果想要使用 GPT-4，确保你的 API 密钥有相应的权限

#### 语言配置说明

`user_language` 字段支持以下值：

1. **中文配置**
```toml
user_language = "zh"
```
生成的提示词：
```text
你是一个代码审查专家，请用中文生成简洁的提交信息。
要求：
1. 只描述修改的主要功能和目的
2. 不要提及具体的修改内容
3. 保持信息简短，一般不超过一行
4. 使用动词开头，描述做了什么
```

2. **英文配置**
```toml
user_language = "en"
```
生成的提示词：
```text
You are a code review expert. Please generate a concise commit message in English.
Requirements:
1. Only describe the main functionality and purpose of the changes
2. Do not mention specific modification details
3. Keep the message brief, typically one line
4. Start with a verb, describing what was done
```

3. **日文配置**
```toml
user_language = "ja"
```
生成的提示词：
```text
あなたはコードレビューの専門家です。簡潔なコミットメッセージを日本語で生成してください。
要件：
1. 変更の主な機能と目的のみを説明する
2. 具体的な変更内容には触れない
3. メッセージは簡潔に、通常1行以内
4. 動詞で始め、何をしたかを説明する
```

4. **其他语言**
如果配置了其他语言代码，将默认使用英文提示词生成英文提交信息。

#### 提交信息示例

不同语言配置下的提交信息示例：

- 中文 (zh)：
  ```
  添加用户认证功能
  优化数据库查询性能
  修复登录验证问题
  ```

- 英文 (en)：
  ```
  Add user authentication feature
  Optimize database query performance
  Fix login validation issue
  ```

- 日文 (ja)：
  ```
  ユーザー認証機能を追加
  データベースクエリのパフォーマンスを最適化
  ログイン認証の問題を修正
  ```

## 使用方式

### 基本使用

```bash
# 提交所有变更
svn-hook commit

# 提交指定文件或目录
svn-hook commit path/to/file1 path/to/file2

# 提交当前目录下的特定文件类型
svn-hook commit *.py *.js

# 提交指定目录下的所有变更
svn-hook commit path/to/directory/
```

### 操作选项

- **自动提交**：
  ```bash
  # 提交所有变更
  svn-hook commit
  # 输入 y 或直接回车

  # 提交指定文件
  svn-hook commit path/to/file1 path/to/file2
  # 输入 y 或直接回车
  ```

- **手动提交**：
  ```bash
  # 提交所有变更
  svn-hook commit
  # 输入 s 获取命令

  # 提交指定文件
  svn-hook commit path/to/file1 path/to/file2
  # 输入 s 获取命令
  ```

- **重新生成**：
  ```bash
  # 对所有变更重新生成提交信息
  svn-hook commit
  # 输入 r 然后输入提示词

  # 对指定文件重新生成提交信息
  svn-hook commit path/to/file1 path/to/file2
  # 输入 r 然后输入提示词
  ```

### 使用示例

```bash
# 示例 1: 提交所有变更
$ svn-hook commit
正在获取变更信息...
正在生成提交信息...

生成的提交信息 (使用模型: gpt-3.5-turbo):
--------------------------------------------------
添加用户认证功能
--------------------------------------------------

选项:
1. 使用此提交信息并自动提交 [y]
2. 显示提交命令 [s]
3. 重新生成 [r]
4. 退出 [n]
请选择 [Y/s/r/n]:

# 示例 2: 提交指定文件
$ svn-hook commit src/auth.py src/config.py
正在获取变更信息...
正在生成提交信息...

生成的提交信息 (使用模型: gpt-3.5-turbo):
--------------------------------------------------
更新认证配置逻辑
--------------------------------------------------

选项:
1. 使用此提交信息并自动提交 [y]
2. 显示提交命令 [s]
3. 重新生成 [r]
4. 退出 [n]
请选择 [Y/s/r/n]:
```

## 注意事项

1. 确保工作目录是有效的 SVN 工作副本
2. 配置文件必须包含有效的 API 密钥
3. 需要网络连接以访问 AI API
4. 生成的提交信息可能需要根据实际情况调整

## 贡献指南

## 许可证

[MIT License](LICENSE)
