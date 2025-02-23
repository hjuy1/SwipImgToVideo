use super::{BigImg, Draw};
use crate::{
    err_new, err_new_image,
    error::{Kind, Result},
};
use image::{DynamicImage, GenericImage, GenericImageView};
use imageproc::{drawing, rect::Rect};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// 定义 Chunk 结构体
#[derive(Serialize, Deserialize)]
pub struct Chunk {
    pic_path: PathBuf,
    text_up: Vec<String>,
    text_down: Vec<String>,
}

// 实现 Chunk 结构体的 Debug trait
impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("pic_path", &self.pic_path.to_str())
            .field("text_up", &self.text_up)
            .field("text_down", &self.text_down)
            .finish()
    }
}

impl Chunk {
    /// 创建一个新的 Chunk 实例
    ///
    /// # Parameters
    ///
    /// * `pic_path` - 图片文件的路径
    /// * `text_up` - 上方文本的向量
    /// * `text_down` - 下方文本的向量
    ///
    /// # Returns
    ///
    /// 如果图片路径有效，则返回一个包含给定数据的 Chunk 实例；否则返回错误。
    ///
    /// # Errors
    ///
    /// * `IoError(NotFound)` - 如果提供的图片路径无效或不存在
    pub fn new(pic_path: PathBuf, text_up: Vec<String>, text_down: Vec<String>) -> Result<Self> {
        if !pic_path.exists() {
            return Err(err_new!(
                Kind::IoError(std::io::ErrorKind::NotFound),
                "Invalid path"
            ));
        }
        Ok(Chunk {
            pic_path,
            text_up,
            text_down,
        })
    }

    /// 绘制 Chunk 数据到一个图像上
    ///
    /// # Parameters
    ///
    /// * `si` - 包含屏幕信息和样式的大图像实例
    ///
    /// # Returns
    ///
    /// 返回一个包含绘制数据的 `DynamicImage` 实例。
    ///
    /// # Errors
    ///
    /// * `ImageError` - 如果打开或处理图片时发生错误
    /// * `TryFromIntError` - 如果在类型转换过程中发生溢出或其他错误
    pub fn draw_data(&self, si: &BigImg) -> Result<DynamicImage> {
        // 解构 BigImg 实例，获取所需的字段
        let BigImg {
            screen,
            width_chunk,
            text_background_color,
            text_color,
            max_scale,
            pic_h,
            text_up_h,
            text_down_h,
            font,
            ..
        } = si;

        // 创建一个新的 `DynamicImage` 实例作为绘制目标
        let mut target = DynamicImage::new_rgba8(*width_chunk, screen.1);

        // 打开并调整图片大小
        let img = image::open(&self.pic_path)
            .map_err(|e| err_new_image!(e))?
            .thumbnail(*width_chunk, *pic_h);
        let (img_w, img_h) = img.dimensions();
        // 将调整好大小的图片复制到目标图像的中心位置
        target
            .copy_from(&img, (width_chunk - img_w) / 2, (pic_h - img_h) / 2)
            .map_err(|e| err_new_image!(e))?;

        // 绘制上下文本的背景框
        let text_up_rect = Rect::at(1, i32::try_from(*pic_h)?).of_size(width_chunk - 1, *text_up_h);
        let text_down_rect =
            Rect::at(1, i32::try_from(pic_h + text_up_h)?).of_size(width_chunk - 1, *text_down_h);
        target.draw_filled_rounded_rect(text_up_rect, 10, text_background_color.0);
        target.draw_filled_rounded_rect(text_down_rect, 10, text_background_color.1);

        // 获取上下文本的内容和长度
        let (text_up, text_down) = (&self.text_up, &self.text_down);
        let (len_up, len_down) = (text_up.len(), text_down.len());
        let (h_up, h_down) = (
            text_up_h / u32::try_from(len_up)?,
            (text_down_h - 30) / u32::try_from(len_down)?,
        );

        // 绘制上文本
        for (i, str) in text_up.iter().enumerate() {
            let high = pic_h + u32::try_from(i)? * h_up;
            target.text_center(
                *text_color,
                Rect::at(10, i32::try_from(high)?).of_size(width_chunk - 20, h_up),
                *max_scale,
                &font,
                str,
            );
        }

        // 绘制下文本
        for (i, str) in text_down.iter().enumerate() {
            let high = pic_h + text_up_h + u32::try_from(i)? * h_down;
            target.text_center(
                *text_color,
                Rect::at(10, i32::try_from(high)?).of_size(width_chunk - 20, h_down),
                *max_scale,
                &font,
                str,
            );
        }

        // 绘制分割线
        drawing::draw_line_segment_mut(
            &mut target,
            (0.0, 10.0),
            (0.0, screen.1 as f32),
            *text_color,
        );

        // 返回绘制完成的图像
        Ok(target)
    }
}
