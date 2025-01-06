# SVN Hook

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
   ```bash
   git clone <repository_url>
   cd svn_hook
   cargo install --path .
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

生成的提交信息:
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

生成的提交信息:
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

欢迎提交 Issue 和 Pull Request 来改进这个工具。

## 许可证

[MIT License](LICENSE)
