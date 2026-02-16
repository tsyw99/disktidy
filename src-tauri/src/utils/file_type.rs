pub fn get_file_type(extension: &str) -> String {
    let ext = extension.to_lowercase();
    match ext.as_str() {
        // 视频文件
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | "3gp" | "m2ts" | "mts" => "视频".to_string(),
        // 音频文件
        "mp3" | "flac" | "wav" | "aac" | "ogg" | "wma" | "m4a" | "opus" | "ape" => "音频".to_string(),
        // 图片文件
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" | "raw" | "heic" | "svg" | "ico" | "psd" | "ai" | "eps" => "图片".to_string(),
        // 压缩包
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "tgz" | "tbz" | "cab" => "压缩包".to_string(),
        // 可执行文件
        "exe" | "msi" | "bat" | "cmd" | "sh" | "ps1" | "vbs" | "jar" | "app" | "dmg" | "pkg" | "deb" | "rpm" | "apk" => "可执行文件".to_string(),
        // 磁盘镜像
        "iso" | "img" | "vhd" | "vhdx" | "vmdk" | "qcow2" => "磁盘镜像".to_string(),
        // 文档
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" | "rtf" | "epub" | "mobi" | "azw" | "azw3" | "md" => "文档".to_string(),
        // 数据库
        "db" | "sqlite" | "sqlite3" | "mdb" | "accdb" | "dbf" => "数据库".to_string(),
        // 文本文件
        "log" | "csv" | "tsv" | "ini" => "文本文件".to_string(),
        // 配置文件
        "json" | "xml" | "yaml" | "yml" | "toml" | "conf" | "config" | "cfg" | "properties" | "env" | "htaccess" => "配置文件".to_string(),
        // 系统文件
        "dll" | "sys" | "drv" | "ocx" | "cpl" | "scr" | "fon" => "系统文件".to_string(),
        // 开发/代码文件
        "c" | "cpp" | "h" | "hpp" | "cs" | "java" | "py" | "rb" | "php" | "go" | "rs" | "swift" | "kt" | "scala" | "r" | "m" | "mm" | "pl" | "lua" | "ts" | "tsx" | "jsx" | "vue" | "sass" | "scss" | "less" | "css" | "html" | "htm" | "js" => "代码文件".to_string(),
        // 字体文件
        "ttf" | "otf" | "woff" | "woff2" | "eot" => "字体文件".to_string(),
        // 备份文件
        "bak" | "old" | "backup" | "orig" => "备份文件".to_string(),
        // 临时文件
        "tmp" | "temp" | "swp" | "swo" => "临时文件".to_string(),
        // 其他
        _ => "其他".to_string(),
    }
}
