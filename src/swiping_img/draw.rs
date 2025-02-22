use ab_glyph::{Font, PxScale, ScaleFont};
use image::Pixel;
use imageproc::{
    definitions::Clamp,
    drawing::{self, Canvas},
    rect::Rect,
};

pub trait Draw: Canvas
where
    <Self::Pixel as Pixel>::Subpixel: Into<f32> + Clamp<f32>,
{
    fn text_center(
        &mut self,
        color: Self::Pixel,
        rect: Rect,
        scale: impl Into<PxScale> + Copy,
        font: &impl Font,
        text: impl AsRef<str>,
    );
    fn draw_filled_rounded_rect(&mut self, rect: Rect, radius: i32, color: Self::Pixel);
}

impl<C> Draw for C
where
    C: Canvas,
    <C::Pixel as Pixel>::Subpixel: Into<f32> + Clamp<f32>,
{
    /// 将文本居中显示在指定矩形区域内
    ///
    /// # Parameters
    /// - `color`: 文本颜色
    /// - `rect`: 文本显示的矩形区域
    /// - `scale`: 文本的缩放比例
    /// - `font`: 使用的字体
    /// - `text`: 要显示的文本内容
    fn text_center(
        &mut self,
        color: Self::Pixel,
        rect: Rect,
        scale: impl Into<PxScale> + Copy,
        font: &impl Font,
        text: impl AsRef<str>,
    ) {
        // 将输入的缩放比例转换为PxScale类型
        let scale: PxScale = scale.into();

        // 将文本按行分割并去除每行的前后空格
        let lines: Vec<&str> = text.as_ref().lines().map(str::trim).collect();

        // 计算文本原始高度
        let row = u32::try_from(lines.len()).unwrap();
        let text_raw_height = row * font.as_scaled(scale.x).height() as u32;

        // 计算文本原始宽度
        let text_raw_width = lines
            .iter()
            .map(|line| drawing::text_size(scale, &font, line).0)
            .max()
            .unwrap_or(0);

        // 解构矩形区域
        let (rect_left, rect_top, rect_width, rect_height) =
            (rect.left(), rect.top(), rect.width(), rect.height());

        // 根据矩形区域和文本原始尺寸计算最终字体大小
        let font_size = if text_raw_width < rect_width && text_raw_height < rect_height {
            scale.x
        } else {
            let x_radio = rect_width as f32 / text_raw_width as f32;
            let y_radio = rect_height as f32 / text_raw_height as f32;
            scale.x * (x_radio.min(y_radio))
        };

        // 重新计算文本高度
        let h = font.as_scaled(font_size).height() as u32;

        // 计算文本顶部位置
        let top_ = rect_top + i32::try_from(rect_height - h * row).unwrap() / 2;

        // 遍历每行文本并绘制
        for (row, line) in lines.iter().enumerate() {
            drawing::draw_text_mut(
                self,
                color,
                rect_left
                    + i32::try_from((rect_width - drawing::text_size(font_size, font, line).0) / 2)
                        .unwrap(),
                top_ + i32::try_from(h).unwrap() * i32::try_from(row).unwrap(),
                font_size,
                font,
                line,
            );
        }
    }

    /// 绘制带有圆角的填充矩形
    ///
    /// Parameters:
    /// * `rect`: 要绘制的矩形区域
    /// * `radius`: 圆角的半径
    /// * `color`: 填充的颜色
    fn draw_filled_rounded_rect(&mut self, rect: Rect, radius: i32, color: Self::Pixel) {
        // 获取矩形的左、右、上、下边界
        let (left, right, top, bottom) = (rect.left(), rect.right(), rect.top(), rect.bottom());

        // 绘制四个圆角
        drawing::draw_filled_circle_mut(self, (left + radius, top + radius), radius, color);
        drawing::draw_filled_circle_mut(self, (left + radius, bottom - radius), radius, color);
        drawing::draw_filled_circle_mut(self, (right - radius, top + radius), radius, color);
        drawing::draw_filled_circle_mut(self, (right - radius, bottom - radius), radius, color);

        // 绘制矩形的顶部和底部，去除圆角部分
        drawing::draw_filled_rect_mut(
            self,
            Rect::at(left, top + radius).of_size(rect.width(), rect.height() - 2 * radius as u32),
            color,
        );
        // 绘制矩形的左侧和右侧，去除圆角部分
        drawing::draw_filled_rect_mut(
            self,
            Rect::at(left + radius, top).of_size(rect.width() - 2 * radius as u32, rect.height()),
            color,
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ab_glyph::FontRef;
    use image::Rgba;

    #[test]
    fn test_text_center() {
        let mut tar = image::DynamicImage::new(700, 700, image::ColorType::Rgb8);
        let font = &FontRef::try_from_slice(include_bytes!("MiSans-Demibold.ttf")).unwrap();
        let red = Rgba([255, 0, 0, 1]);
        let text = "this a test\n this is a new line\n this a line 3";

        let rect = Rect::at(50, 202).of_size(300, 190);
        tar.text_center(red, rect, 170.0, font, text);

        let output_path = "./src/test1.png";
        tar.save_with_format(output_path, image::ImageFormat::Png)
            .unwrap();
    }

    #[test]
    fn test_draw_filled_rounded_rect() {
        let mut tar = image::DynamicImage::new(200, 200, image::ColorType::Rgb8);
        tar.draw_filled_rounded_rect(Rect::at(50, 50).of_size(100, 100), 20, Rgba([255, 0, 0, 1]));
        tar.save("./src/test_2.png").unwrap();
    }
}
