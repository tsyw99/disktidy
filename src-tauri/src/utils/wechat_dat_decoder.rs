use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

const XOR_KEY_SEARCH_BYTES: usize = 8;
const MIN_FILE_SIZE: u64 = 10;

#[derive(Debug, Clone)]
pub struct WeChatImageInfo {
    pub original_format: ImageFormat,
    pub xor_key: u8,
    pub file_size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    Bmp,
    WebP,
    Unknown,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::WebP => "webp",
            ImageFormat::Unknown => "dat",
        }
    }
    
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Bmp => "image/bmp",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Unknown => "application/octet-stream",
        }
    }
}

static IMAGE_SIGNATURES: &[(&[u8], ImageFormat)] = &[
    (&[0xFF, 0xD8, 0xFF], ImageFormat::Jpeg),
    (&[0x89, 0x50, 0x4E, 0x47], ImageFormat::Png),
    (&[0x47, 0x49, 0x46, 0x38], ImageFormat::Gif),
    (&[0x42, 0x4D], ImageFormat::Bmp),
    (&[0x52, 0x49, 0x46, 0x46], ImageFormat::WebP),
];

pub struct WeChatDatDecoder;

impl WeChatDatDecoder {
    pub fn is_dat_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase() == "dat")
            .unwrap_or(false)
    }
    
    pub fn detect_xor_key(data: &[u8]) -> Option<u8> {
        if data.len() < XOR_KEY_SEARCH_BYTES {
            return None;
        }
        
        for (signature, _) in IMAGE_SIGNATURES {
            if signature.len() <= data.len() {
                let potential_key = data[0] ^ signature[0];
                
                let mut matches = true;
                for (i, &sig_byte) in signature.iter().enumerate() {
                    if data[i] ^ potential_key != sig_byte {
                        matches = false;
                        break;
                    }
                }
                
                if matches {
                    return Some(potential_key);
                }
            }
        }
        
        if data.len() >= 2 {
            for test_key in 0x00..=0xFF {
                let decoded_first = data[0] ^ test_key;
                let decoded_second = data[1] ^ test_key;
                
                if decoded_first == 0xFF && decoded_second == 0xD8 {
                    return Some(test_key);
                }
                if decoded_first == 0x89 && decoded_second == 0x50 {
                    return Some(test_key);
                }
                if decoded_first == 0x47 && decoded_second == 0x49 {
                    return Some(test_key);
                }
                if decoded_first == 0x42 && decoded_second == 0x4D {
                    return Some(test_key);
                }
            }
        }
        
        None
    }
    
    pub fn detect_image_format(decrypted_header: &[u8]) -> ImageFormat {
        for (signature, format) in IMAGE_SIGNATURES {
            if signature.len() <= decrypted_header.len() {
                if decrypted_header.starts_with(signature) {
                    return format.clone();
                }
            }
        }
        ImageFormat::Unknown
    }
    
    pub fn analyze_dat_file(path: &Path) -> Option<WeChatImageInfo> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        
        let metadata = std::fs::metadata(path).ok()?;
        let file_size = metadata.len();
        
        if file_size < MIN_FILE_SIZE {
            return None;
        }
        
        let mut header = [0u8; XOR_KEY_SEARCH_BYTES];
        reader.read_exact(&mut header).ok()?;
        
        let xor_key = Self::detect_xor_key(&header)?;
        
        let decrypted_header: Vec<u8> = header.iter().map(|&b| b ^ xor_key).collect();
        let original_format = Self::detect_image_format(&decrypted_header);
        
        Some(WeChatImageInfo {
            original_format,
            xor_key,
            file_size,
        })
    }
    
    pub fn decrypt_dat_file(path: &Path) -> Option<(Vec<u8>, ImageFormat)> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        
        let metadata = std::fs::metadata(path).ok()?;
        let file_size = metadata.len() as usize;
        
        if file_size < MIN_FILE_SIZE as usize {
            return None;
        }
        
        let mut data = Vec::with_capacity(file_size);
        reader.read_to_end(&mut data).ok()?;
        
        let xor_key = Self::detect_xor_key(&data)?;
        
        let decrypted: Vec<u8> = data.iter().map(|&b| b ^ xor_key).collect();
        let format = Self::detect_image_format(&decrypted);
        
        Some((decrypted, format))
    }
    
    pub fn decrypt_dat_file_chunked(path: &Path, chunk_size: usize) -> Option<(Vec<u8>, ImageFormat)> {
        let file = File::open(path).ok()?;
        let metadata = std::fs::metadata(path).ok()?;
        let file_size = metadata.len();
        
        if file_size < MIN_FILE_SIZE {
            return None;
        }
        
        let mut reader = BufReader::new(file);
        
        let mut header = [0u8; XOR_KEY_SEARCH_BYTES];
        reader.read_exact(&mut header).ok()?;
        
        let xor_key = Self::detect_xor_key(&header)?;
        
        let decrypted_header: Vec<u8> = header.iter().map(|&b| b ^ xor_key).collect();
        let format = Self::detect_image_format(&decrypted_header);
        
        let mut decrypted = Vec::with_capacity(file_size as usize);
        decrypted.extend_from_slice(&decrypted_header);
        
        reader.seek(SeekFrom::Start(XOR_KEY_SEARCH_BYTES as u64)).ok()?;
        
        let mut buffer = vec![0u8; chunk_size];
        loop {
            let bytes_read = reader.read(&mut buffer).ok()?;
            if bytes_read == 0 {
                break;
            }
            
            for byte in &buffer[..bytes_read] {
                decrypted.push(*byte ^ xor_key);
            }
        }
        
        Some((decrypted, format))
    }
    
    pub fn decrypt_to_file(source: &Path, target: &Path) -> Option<ImageFormat> {
        let (decrypted, format) = Self::decrypt_dat_file(source)?;
        std::fs::write(target, decrypted).ok()?;
        Some(format)
    }
}

pub fn is_wechat_encrypted_image(path: &Path) -> bool {
    WeChatDatDecoder::is_dat_file(path) && WeChatDatDecoder::analyze_dat_file(path).is_some()
}

pub fn get_decrypted_image_format(path: &Path) -> Option<ImageFormat> {
    WeChatDatDecoder::analyze_dat_file(path).map(|info| info.original_format)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_xor_key_jpeg() {
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        let key = 0xAB;
        let encrypted: Vec<u8> = jpeg_header.iter().map(|&b| b ^ key).collect();
        
        let detected = WeChatDatDecoder::detect_xor_key(&encrypted);
        assert_eq!(detected, Some(key));
    }
    
    #[test]
    fn test_detect_xor_key_png() {
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let key = 0x55;
        let encrypted: Vec<u8> = png_header.iter().map(|&b| b ^ key).collect();
        
        let detected = WeChatDatDecoder::detect_xor_key(&encrypted);
        assert_eq!(detected, Some(key));
    }
    
    #[test]
    fn test_detect_image_format() {
        assert_eq!(WeChatDatDecoder::detect_image_format(&[0xFF, 0xD8, 0xFF]), ImageFormat::Jpeg);
        assert_eq!(WeChatDatDecoder::detect_image_format(&[0x89, 0x50, 0x4E, 0x47]), ImageFormat::Png);
        assert_eq!(WeChatDatDecoder::detect_image_format(&[0x47, 0x49, 0x46, 0x38]), ImageFormat::Gif);
        assert_eq!(WeChatDatDecoder::detect_image_format(&[0x42, 0x4D]), ImageFormat::Bmp);
    }
}
