<h1 align="center">SyMark - 思源笔记网页转换器</h1>
SyMark 是一个开源项目,可一键将思源笔记内容转换到您的网站。笔记内的复杂内容(双向链接、标签和任何字体格式)在网页上自然的显示,所见即所得。

![symark](https://github.com/user-attachments/assets/5e5eedc5-412e-4635-a768-64d46e86b75e)

# 总览
SyMark 处理来自思源笔记的 .sy 文件，为每个注释、标签集合和主页生成一个包含 HTML 页面的静态网站，它保留了笔记的结构和格式,同时创建了干净、响应迅速的 Web 界面。

# 功能介绍
将思源笔记转换为 HTML 网页
保持笔记内的的双向链接与跨文档引用
支持带有专用标签页的标签
自动生成相关目录
渲染 Markdown 格式、代码块和其他思源笔记功能
支持使用figure/figcaption元素添加图片说明
自定义索引页面支持
自动删除空格字符
高性能处理超大内容笔记(1,000,000+）
没有跟踪器或遥测

# 安装流程

## 前提条件
Rust (请更新到最新版本)
思源笔记导出的 .sy 文件

# 运行SyMark
1.解压`NotebookName.sy.zip`到SyMark的`input`文件夹
2.为Symark选择一种主题
#使用默认主题
`cargo run`
#使用特定主题
`cargo run my-theme`
注解： 使用默认主题无需输入主题名称
3.生成的网页文件会保存在`output`文件夹
4.打开`output/index.html`预览你生成网页


该程序将显示生成过程的相关信息，包括：

处理的笔记数量
已知的标签
生成的HTML文件
构建时间统计

自定义与主题
Symark支持多个主题，在生成页面时可选用

项目的程序结构
```
symark/
├── input/              # 输入文件夹
│   ├── assets/         # 资源文件夹
│   ├── [note-id]/      # 思源笔记ID
│   └── [note-id].sy    # 思源笔记文件夹 (JSON 格式)
├── themes/             # 主题文件夹
│   ├── default/        # 默认主题(留作备用)
│   │   ├── page.html   # HTML 模板
│   │   ├── styles.css  # CSS 模板
│   │   └── graph.html  # Graph 模板
│   └── [theme-name]/   # 自定义主题
└── output/             # 输出文件夹
```
## 主题与模板
自定义主题需要包含三类文件page.html styles.css graph.html
缺少的文件会自动从默认主题文件夹中调用

# 自定义与主题

## 自定义主页

在思源笔记建立带有主页标签的笔记，SyMark则会使用该笔记作为主页生成index.html
例如，思源笔记文件夹的“20250506164324-csw026m.sy”带有“index”标签，将作为主页使用。系统还将生成包含所有笔记链接的“all.html”页面。

风格
通过编辑CSS文件自定义风格

标签
在思源笔记中添加的标签，将在生成的网站中转化为可浏览的分类集合。对于每个独特的标签，SyMark都会创建一个专属页面，列出所有带有该标签的笔记。

导航栏
生成的网站文件会包含:

index.html: 主页 (自定义或默认的列表)
all.html: 列表中包含所有笔记
tag_[tagname].html: 单独为[tagname]生成的网页文件 (e.g., tag_Features.html)
[note-id].html: 单独为[note-id]笔记生成的网页文件 (e.g., 20250506164324-csw026m.html)
graph.html: 为笔记生成的关联网页文件，可访问上述内容

故障排除
1确认你的.sy为json格式
2检查相关资源是否在input/assets/下
3确认用户组对output文件夹有读取权限
4确认笔记ID为思源笔记格式(YYYYMMDDhhmmss-xxxxx)
5如果遇到资源缺失，请检查控制台出现的警告信息

常见问题
图片缺失：确保图片资源在/assets/文件夹
链接失效：检查输入文件夹中是否存在引用的笔记ID
格式问题：确认你的思源笔记使用了支持的格式
无效图片说明：请在思源笔记中为图片添加说明

性能优化
SyMark为超大内容的笔记提供专门优化：

眨眼见处理数百条笔记
自动移除空格字符
极低内存占用
主题化模板支持便捷定制

## 软件许可
```
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.
```
For more information, please refer to <https://unlicense.org/>

## Acknowledgements

SyMark is designed to work with [SiYuan](https://github.com/siyuan-note/siyuan), an excellent open-source personal knowledge management system.
