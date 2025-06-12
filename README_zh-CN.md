[English Version](README.md)

# SVN AI Commit Hook - 智能 SVN 提交信息助手

`SVN AI Commit Hook` 是一个强大的 Subversion (SVN) 预提交钩子脚本，它利用人工智能（AI）的强大能力，根据您暂存的更改自动生成规范、清晰且富有信息的提交说明。告别手动编写提交信息的繁琐，让 AI 助您提升版本控制效率与质量。

## ✨ 核心特性

- **🤖 AI 驱动的提交信息生成**：在您执行 `svn commit` 命令时自动触发（如果未通过 `-m` 或 `-F` 参数提供提交信息），分析本次提交的差异（diff），并调用 AI 模型（如 GPT 系列）生成高质量的提交信息。
- **📝 规范化与一致性**：确保团队的提交信息风格统一，遵循预设的格式和语言要求（例如，中文或英文，以及特定的提交类型前缀）。
- **⏱️ 节省时间，提升效率**：大幅减少开发人员在思考和撰写提交信息上花费的时间，让他们更专注于编码本身。
- **💡 上下文感知**：AI 能够理解代码更改的意图，生成比通用模板更具描述性的提交信息。
- **⚙️ 灵活配置**：通过共享的配置文件 `~/.config/commit_crafter/config.toml`，您可以轻松设置 API 密钥、AI 模型、以及期望的提交信息语言等。
- **💬 交互式确认**：在自动提交前，会展示 AI 生成的提交信息，并允许您**确认使用**、**手动编辑**或**取消本次提交**，确保您对最终的提交信息拥有完全控制权。

## 📋 环境准备与配置

本钩子脚本实际上是一个 Rust 编译生成的可执行程序，它负责与 AI 服务交互并处理用户交互。

1.  **Rust 可执行程序 (`svn-hook`)**：
    确保您已经通过项目提供的 `install.sh` 脚本编译并安装了 `svn-hook` 程序。该脚本通常会将可执行文件安装到 `~/.cargo/bin/` 目录下，请确保此目录已添加到系统的 `PATH` 环境变量中。

2.  **创建并配置 `config.toml`**：
    本工具与 `File Organizer` 共用位于 `~/.config/commit_crafter/config.toml` 的配置文件。
    如果您尚未配置，请执行以下操作：
    ```bash
    # 创建目录 (如果尚不存在)
    mkdir -p ~/.config/commit_crafter
    
    # 创建配置文件 (如果尚不存在)
    touch ~/.config/commit_crafter/config.toml
    ```
    在 `config.toml` 文件中填入以下内容：
    ```toml
    # 您的 OpenAI API 密钥
    openai_api_key = "sk-your-key-here"

    # OpenAI API 的基础 URL (如果您使用代理，请修改此处)
    openai_url = "https://api.openai.com"

    # 用于生成提交信息的 AI 模型
    openai_model = "gpt-4-turbo" # 或其他您偏好的模型

    # (可选) 指定 AI 生成提交信息时使用的语言
    user_language = "Chinese" # 或 "English"
    ```

## 🔧 SVN 钩子配置

要使此 AI 提交助手生效，您需要在您的 SVN 仓库中配置一个 `pre-commit` 钩子。此钩子脚本将调用我们编译好的 `svn-hook` Rust 程序。

1.  **定位钩子目录**：
    对于服务器端钩子（推荐方式），请进入您的 SVN 服务器上仓库的 `hooks` 目录。
    对于客户端钩子（部分 SVN 版本和配置可能不支持或不推荐），它可能位于工作副本的 `.svn/hooks/` 目录下（但这通常用于本地校验，而非集中控制）。

2.  **创建 `pre-commit` 脚本**：
    在钩子目录中，创建一个名为 `pre-commit` 的可执行脚本 (例如，在 Linux/macOS 上)。Windows 上可能是 `pre-commit.bat`。

3.  **`pre-commit` 脚本内容示例 (Linux/macOS)**：
    ```bash
    #!/bin/bash

    REPOS="$1"  # 仓库路径 (由 SVN 传递)
    TXN="$2"    # 事务名称 (由 SVN 传递)

    # svn-hook 可执行文件的路径 (如果已加入PATH，则直接写 svn-hook)
    SVN_HOOK_EXEC="svn-hook"

    # 检查用户是否通过 -m 或 -F 参数提供了提交信息
    # 如果提供了，svnlook log "$REPOS" -t "$TXN" 会返回该信息
    # svn-hook 程序内部会处理此逻辑：如果用户已提供信息，则直接退出并允许提交。

    # 创建一个临时文件来存储 svn-hook 的输出 (即最终的提交信息)
    TMP_MSG_FILE=$(mktemp)

    # 调用 svn-hook Rust 程序
    # 它会与用户交互，如果用户确认，则将最终的提交信息打印到 stdout
    # 错误信息会打印到 stderr
    if $SVN_HOOK_EXEC --repo-path "$REPOS" --txn "$TXN" > "$TMP_MSG_FILE"; then
        # svn-hook 成功退出 (用户确认了提交信息)
        # 使用临时文件的内容设置 svn:log 属性
        if [ -s "$TMP_MSG_FILE" ]; then # 确保临时文件有内容
            svnlook propset --txn "$TXN" "$REPOS" svn:log -F "$TMP_MSG_FILE"
            rm "$TMP_MSG_FILE"
            exit 0 # 允许提交
        else
            # 理论上不应发生：成功退出但没有提交信息
            echo "错误：svn-hook 成功执行但未提供提交信息。" >&2
            rm "$TMP_MSG_FILE"
            exit 1 # 阻止提交
        fi
    else
        # svn-hook 异常退出 (用户取消操作、AI 调用失败等)
        # Rust 程序应已将错误信息输出到 stderr，SVN 客户端会显示它
        rm "$TMP_MSG_FILE"
        exit 1 # 阻止提交
    fi
    ```
    **重要提示**：
    *   确保 `pre-commit` 脚本具有可执行权限 (`chmod +x pre-commit`)。
    *   `SVN_HOOK_EXEC` 变量应指向您编译安装的 `svn-hook` Rust 程序。如果它在系统的 `PATH` 中，直接写 `svn-hook` 即可。
    *   此脚本的核心逻辑是：调用 `svn-hook` 程序，捕获其标准输出（即用户确认后的提交信息），然后用这个信息设置当前 SVN 事务的 `svn:log` 属性。

## 💡 使用流程

1.  在您的 SVN 工作副本中进行代码修改。
2.  执行 `svn add ...` （如果需要添加新文件）然后执行 `svn commit`。
3.  当您执行 `svn commit` 时：
    *   如果您**未**通过 `-m "message"` 或 `-F file` 提供提交信息，`pre-commit` 钩子将触发 `svn-hook` 程序。
    *   如果您**已**提供提交信息，`svn-hook` 程序会检测到这一点并直接允许提交，跳过 AI 生成步骤。
4.  如果触发了 AI：AI 将分析您的更改并生成建议的提交信息。
5.  您将在终端看到建议的提交信息，并可以选择：
    - **(y)es/是**: 直接使用 AI 生成的信息进行提交。
    - **(e)dit/编辑**: 在您默认的文本编辑器中修改 AI 生成的信息，保存并关闭编辑器后提交。
    - **(n)o/否**: 取消本次提交。

---
*本工具由 Cascade (世界一流的 AI 编程助手) 协助开发。*
