pub mod chunk;
mod draw;

use crate::{
    err_new, err_new_image, err_new_io, err_new_tryfrom,
    error::{Kind, Result},
};
use ab_glyph::FontVec;
pub use chunk::Chunk;
use image::{DynamicImage, GenericImage, Rgba};
#[allow(unused_imports)]
use std::{
    fmt::{self, Debug},
    path::{Path, PathBuf},
    process::Command,
};

/// 大图像处理结构体
///
/// 该结构体用于处理大图像，通过将图像分割成多个块来实现，
/// 并提供了屏幕适配、文本渲染和视频生成的相关参数。
///
/// # Parameters
///
/// * `work_dir`: 图像操作的工作路径。
/// * `chunks`: 图像块数据数组的引用。
/// * `screen`: 显示图像的屏幕分辨率（宽度，高度）。
/// * `step`: 每次处理图像块的数量。
/// * `width_chunk`: 每个图像块的宽度。
/// * `overlap`: 重叠图像块数，即屏幕能同时显示图像块的数量。
/// * `text_background_color`: 文本的背景颜色，包括上下两种颜色。
/// * `text_color`: 文本的颜色。
/// * `max_scale`: 字体的最大缩放因子。
/// * `pic_h`: 图像块中的图片区域高度。
/// * `text_up_h`: 图像块中的上方文本的高度。
/// * `text_down_h`: 图像块中的下方文本的高度。
/// * `font`: 文本渲染使用的字体。
/// * `video_cover_time`: 视频封面图像的持续时间。
/// * `video_ending_time`: 视频结束图像的持续时间。
/// * `video_background_color`: 视频的背景颜色，以字符串表示。
/// * `video_swip_speed`: 视频的滑动速度，用视频滑动 `width_chunk` 所需的秒数表示。
/// * `video_fps`: 视频的帧率（每秒帧数）。
pub struct BigImg<'a> {
    work_dir: PathBuf,
    chunks: &'a [Chunk],
    screen: (u32, u32),
    step: u32,
    width_chunk: u32,
    overlap: u32,
    text_background_color: (Rgba<u8>, Rgba<u8>),
    text_color: Rgba<u8>,
    max_scale: f32,
    pic_h: u32,
    text_up_h: u32,
    text_down_h: u32,
    font: FontVec,
    video_cover_time: u32,
    video_ending_time: u32,
    video_background_color: String,
    video_swip_speed: u32,
    video_fps: u32,
}

impl<'a> BigImg<'a> {
    /// 创建一个新的 `BigImg` 实例。
    ///
    /// # Parameters
    /// - `work_dir`: 工作路径，用于保存生成的文件。
    /// - `chunks`: 图像块的引用切片。
    ///
    /// # Results
    /// 返回一个新的 `BigImg` 实例。
    ///
    /// # Panics
    /// 如果构建过程中发生错误（例如无效参数），则expect("BigImg new failed")。
    ///
    #[must_use]
    pub fn new(work_dir: &Path, chunks: &'a [Chunk]) -> BigImg<'a> {
        BigImgBuilder::new(work_dir, chunks)
            .build()
            .expect("BigImg new failed")
    }

    /// 创建一个新的 `BigImgBuilder` 实例。
    ///
    /// # Parameters
    /// - `work_dir`: 工作路径，用于保存生成的文件。
    /// - `chunks`: 图像块的引用切片。
    ///
    /// # Results
    /// 返回一个新的 `BigImgBuilder` 实例。
    ///
    #[must_use]
    pub fn builder(work_dir: &Path, chunks: &'a [Chunk]) -> BigImgBuilder<'a> {
        BigImgBuilder::new(work_dir, chunks)
    }
}

impl BigImg<'_> {
    /// 将图像块分割成多个子块。
    ///
    /// # Results
    /// 返回一个包含分割后子块的向量。
    ///
    fn divide(&self) -> Vec<&[Chunk]> {
        let len = self.chunks.len();
        (0..len - self.overlap as usize)
            .step_by((self.step - self.overlap) as usize)
            .map(|i| &self.chunks[i..(i + self.step as usize).min(len)])
            .collect()
    }

    /// 将多个图像块组合成一个完整的图像并保存。
    ///
    /// # Parameters
    /// - `chunk`: 要组合的图像块切片。
    /// - `save_name`: 组合后的图像保存路径。
    ///
    /// # Results
    /// 如果成功，则返回 `Ok(())`；如果失败，则返回 `Err`。
    ///
    /// # Errors
    /// - 如果 `chunk` 为空，则返回 `Err`。
    /// - 如果图像处理或保存过程中发生错误，则返回 `Err`。
    ///
    fn combain_chunk(&self, chunk: &[Chunk], save_name: &Path) -> Result<()> {
        if chunk.is_empty() {
            return Err(err_new!(Kind::Other, "Empty chunk"));
        }

        let len = u32::try_from(chunk.len()).map_err(|e| err_new_tryfrom!(e))?;
        let mut target = DynamicImage::new_rgba8(len * self.width_chunk, self.screen.1);

        // 将每张图片绘制到目标图像中
        for (i, item) in chunk.iter().enumerate() {
            let img = item.draw_data(self).map_err(|e| err_new_image!(e))?;
            target
                .copy_from(&img, u32::try_from(i)? * self.width_chunk, 0)
                .map_err(|e| err_new_image!(e))?;
        }

        // 保存组合后的图像
        target
            .save(self.work_dir.join(save_name))
            .map_err(|e| err_new_image!(e))?;

        println!("{save_name:?} successed");
        Ok(())
    }

    /// 生成视频封面或结尾视频。
    ///
    /// # Parameters
    /// - `chunk`: 要使用的图像块切片。
    /// - `pic_name`: 生成的图片名称。
    /// - `video_time`: 视频时长（秒）。
    ///
    /// # Results
    /// 如果成功，则返回生成的视频文件路径；如果失败，则返回 `Err`。
    ///
    /// # Errors
    /// - 如果图像处理或保存过程中发生错误，则返回 `Err`。
    /// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
    ///
    fn generate_endpoint_video(
        &self,
        chunk: &[Chunk],
        pic_name: &Path,
        video_time: u32,
    ) -> Result<PathBuf> {
        let video_name = pic_name.with_extension("mp4");
        self.combain_chunk(chunk, pic_name)?;
        self.ffmpeg(&[
            "-r",
            "1",
            "-loop",
            "1",
            "-i",
            pic_name.to_str().unwrap(),
            "-filter_complex",
            &format!(
                "color={}:s={}x{}:r={}[bg];[bg][0]overlay=shortest=1",
                self.video_background_color, self.screen.0, self.screen.1, self.video_fps
            ),
            "-preset",
            "fast",
            "-t",
            &video_time.to_string(),
            "-y",
            video_name.to_str().unwrap(),
        ])?;
        println!("{video_name:?} successed");
        Ok(video_name)
    }

    /// 生成中间部分的视频。
    ///
    /// # Parameters
    /// - `chunk`: 要使用的图像块切片。
    /// - `pic_name`: 生成的图片名称。
    ///
    /// # Results
    /// 如果成功，则返回生成的视频文件路径；如果失败，则返回 `Err`。
    ///
    /// # Errors
    /// - 如果图像处理或保存过程中发生错误，则返回 `Err`。
    /// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
    ///
    fn generate_mid_video(&self, chunk: &[Chunk], pic_name: &Path) -> Result<PathBuf> {
        self.combain_chunk(chunk, pic_name)?;
        let video_name = pic_name.with_extension("mp4");

        let adjust_len = u32::try_from(chunk.len())? - self.overlap;
        let run_seconds = self.video_swip_speed * adjust_len + 1;
        let speed = self.width_chunk / self.video_swip_speed;

        self.ffmpeg(&[
            "-r",
            "1",
            "-loop",
            "1",
            "-t",
            &run_seconds.to_string(),
            "-i",
            pic_name.to_str().unwrap(),
            "-filter_complex",
            &format!(
                "color={}:s={}x{}:r={}[bg];[bg][0]overlay=x=-t*{speed}:shortest=1",
                self.video_background_color, self.screen.0, self.screen.1, self.video_fps
            ),
            "-preset",
            "fast",
            "-y",
            video_name.to_str().unwrap(),
        ])?;
        println!("{video_name:?} successed");
        Ok(video_name)
    }

    /// 组合所有图像块并生成最终视频。
    ///
    /// # Parameters
    /// - `save_name`: 最终视频文件名。
    ///
    /// # Results
    /// 如果成功，则返回 `Ok(())`；如果失败，则返回 `Err`。
    ///
    /// # Errors
    /// - 如果图像处理或保存过程中发生错误，则返回 `Err`。
    /// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
    ///
    pub fn combain(&self, save_name: &str) -> Result<()> {
        let chunks = self.divide();
        let mut results = Vec::with_capacity(chunks.len() + 2);

        let cover_pic_name = Path::new("cover.png");
        let cover_video_name = self.generate_endpoint_video(
            &self.chunks[..self.overlap as usize],
            cover_pic_name,
            self.video_cover_time,
        )?;
        results.push(cover_video_name);

        for (index, &chunk) in chunks.iter().enumerate() {
            let mid_pic_name = format!("{index:0>2}.png");
            let mid_pic_name = Path::new(&mid_pic_name);
            let mid_video_name = self.generate_mid_video(chunk, mid_pic_name)?;

            results.push(mid_video_name);
        }

        let ending_pic_name = Path::new("ending.png");
        let ending_video_name = self.generate_endpoint_video(
            &self.chunks[(self.chunks.len() - self.overlap as usize)..],
            ending_pic_name,
            self.video_ending_time,
        )?;
        results.push(ending_video_name);

        let result_str =
            results
                .iter()
                .fold(String::with_capacity(results.len() * 10), |mut init, s| {
                    init.push_str("file ");
                    init.push_str(&s.to_string_lossy());
                    init.push('\n');
                    init
                });

        let list_file = self.work_dir.join("list.txt");
        std::fs::write(&list_file, result_str)?;

        self.ffmpeg(&[
            "-f",
            "concat",
            "-i",
            list_file.to_str().unwrap(),
            "-c",
            "copy",
            "-y",
            save_name,
        ])?;

        println!("{save_name} successed");

        // 清理临时文件
        for result in results {
            let _ = std::fs::remove_file(self.work_dir.join(result));
        }

        Ok(())
    }

    /// 执行带有指定参数的FFmpeg命令
    ///
    /// # Parameters
    /// - `&self` - 包含工作路径配置的结构体实例引用
    /// - `args` - 传递给ffmpeg命令行工具的字符串参数切片
    ///
    /// # Results
    /// - 成功时返回Ok(())，失败时返回包含上下文信息的Err
    ///
    /// # Errors
    /// - 无法执行ffmpeg命令时返回IO错误
    /// - ffmpeg进程返回非零状态码时打印stderr到控制台并返回Other类型错误
    ///
    #[allow(unused)]
    fn ffmpeg(&self, args: &[&str]) -> Result<()> {
        let command = Command::new("ffmpeg")
            .current_dir(&self.work_dir)
            .args(args)
            .output()?;
        if !command.status.success() {
            println!("{}", String::from_utf8(command.stderr).unwrap());
            return Err(err_new!(Kind::Other, "FFmpeg command failed"));
        }
        Ok(())
    }
}

impl Debug for BigImg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BigImg")
            .field("datas_len", &self.chunks.len())
            .field("screen", &self.screen)
            .field("step", &self.step)
            .field("width_chunk", &self.width_chunk)
            .field("overlap", &self.overlap)
            .field("text_background_color", &self.text_background_color)
            .field("text_color", &self.text_color)
            .field("max_scale", &self.max_scale)
            .field("pic_h", &self.pic_h)
            .field("text_up_h", &self.text_up_h)
            .finish()
    }
}

pub struct BigImgBuilder<'a> {
    work_dir: PathBuf,
    chunks: &'a [Chunk],
    screen: (u32, u32),
    pub step: u32,
    width_chunk: u32,
    text_background_color: (Rgba<u8>, Rgba<u8>),
    text_color: Rgba<u8>,
    max_scale: f32,
    pic_h: u32,
    text_up_h: u32,
    font: Option<FontVec>,
    video_cover_time: u32,
    video_ending_time: u32,
    video_background_color: String,
    video_swip_speed: u32,
    video_fps: u32,
}

impl<'a> BigImgBuilder<'a> {
    /// 创建一个新的 `BigImgBuilder` 实例。
    ///
    /// # Parameters
    /// - `work_dir`: 工作路径，用于保存生成的文件。
    /// - `chunks`: 图像块的引用切片。
    ///
    /// # Results
    /// 返回一个新的 `BigImgBuilder` 实例。
    ///
    #[must_use]
    pub fn new(work_dir: &Path, chunks: &'a [Chunk]) -> BigImgBuilder<'a> {
        Self {
            work_dir: work_dir.to_path_buf(),
            chunks,
            screen: (1920, 1080),
            step: 100,
            width_chunk: 480,
            text_background_color: (Rgba([23, 150, 235, 255]), Rgba([44, 85, 153, 255])),
            text_color: Rgba([255, 255, 255, 255]),
            max_scale: 120.0,
            pic_h: 520,
            text_up_h: 214,
            font: None,
            video_cover_time: 3,
            video_ending_time: 3,
            video_background_color: String::from("white"),
            video_swip_speed: 3,
            video_fps: 60,
        }
    }

    /// 构建 `BigImg` 实例。
    ///
    /// # Parameters
    /// 无。
    ///
    /// # Results
    /// 如果成功，则返回一个新的 `BigImg` 实例；如果失败，则返回 `Err`。
    ///
    /// # Errors
    /// - 如果 `chunks` 为空，则返回 `Err`。
    /// - 如果 `pic_h` 大于屏幕高度，则返回 `Err`。
    /// - 如果屏幕宽度不能被 `width_chunk` 整除，则返回 `Err`。
    /// - 如果字体加载失败，则返回 `Err`。
    ///
    pub fn build(&mut self) -> Result<BigImg<'a>> {
        if self.chunks.is_empty() {
            return Err(err_new!(Kind::BigImgBuilderError, "chunks data is empty"));
        }
        if self.pic_h > self.screen.1 {
            return Err(err_new!(
                Kind::BigImgBuilderError,
                &format!(
                    "err:\n{},\n{}\n pic_h > height_screen; {} > {}",
                    file!(),
                    line!(),
                    self.pic_h,
                    self.screen.1
                )
            ));
        }
        if self.screen.0 % self.width_chunk != 0 {
            return Err(err_new!(
                Kind::BigImgBuilderError,
                &format!(
                    "err: width_screen % width_chunk != 0; {} % {} != 0",
                    self.screen.0, self.width_chunk
                )
            ));
        }
        self.step = self.step.min(u32::try_from(self.chunks.len()).unwrap_or(0));
        Ok(BigImg {
            work_dir: self.work_dir.clone(),
            chunks: self.chunks,
            screen: self.screen,
            step: self.step,
            width_chunk: self.width_chunk,
            overlap: self.screen.0 / self.width_chunk,
            text_background_color: self.text_background_color,
            text_color: self.text_color,
            max_scale: self.max_scale,
            pic_h: self.pic_h,
            text_up_h: self.text_up_h,
            text_down_h: self.screen.1 - self.pic_h - self.text_up_h,
            font: self.font.take().unwrap_or({
                let font_buf = std::fs::read("./src/swiping_img/MiSans-Demibold.ttf")
                    .map_err(|e| err_new_io!(e))?;
                FontVec::try_from_vec(font_buf)
                    .map_err(|e| err_new!(Kind::InvalidFont, &e.to_string()))?
            }),
            video_cover_time: self.video_cover_time,
            video_ending_time: self.video_ending_time,
            video_background_color: self.video_background_color.clone(),
            video_swip_speed: self.video_swip_speed,
            video_fps: self.video_fps,
        })
    }
}

impl BigImgBuilder<'_> {
    /// 设置屏幕分辨率。
    ///
    /// # Parameters
    /// - `screen`: 屏幕分辨率元组 `(宽度, 高度)`。
    ///
    /// # Results
    /// 返回可变引用 `&mut Self`，以便链式调用。
    ///
    /// # Panics
    /// 如果屏幕宽高为零，则会触发断言失败。
    ///
    pub fn screen(&mut self, screen: (u32, u32)) -> &mut Self {
        assert!(
            screen.0 != 0 && screen.1 != 0,
            "Screen dimensions must be non-zero."
        );
        self.screen = screen;
        self
    }

    /// 设置步长
    ///
    /// # Parameters
    /// - `step`: 步长值，必须是非零值
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    /// # Panics
    /// - 如果 `step` 为零，程序将 panic
    ///
    pub fn step(&mut self, step: u32) -> &mut Self {
        assert_ne!(step, 0, "Step must be non-zero.");
        self.step = step;
        self
    }

    /// 设置宽度块大小
    ///
    /// # Parameters
    /// - `width_chunk`: 宽度块大小，必须是非零值
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    /// # Panics
    /// - 如果 `width_chunk` 为零，程序将 panic
    ///
    pub fn width_chunk(&mut self, width_chunk: u32) -> &mut Self {
        assert_ne!(width_chunk, 0, "Width chunk must be non-zero.");
        self.width_chunk = width_chunk;
        self
    }

    /// 设置文本颜色
    ///
    /// # Parameters
    /// - `text_color`: 文本颜色，使用 `Rgba<u8>` 类型表示
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    pub fn text_color(&mut self, text_color: Rgba<u8>) -> &mut Self {
        self.text_color = text_color;
        self
    }

    /// 设置文本背景颜色
    ///
    /// # Parameters
    /// - `color`: 文本背景颜色，使用 `(Rgba<u8>, Rgba<u8>)` 类型表示
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    pub fn text_background_color(&mut self, color: (Rgba<u8>, Rgba<u8>)) -> &mut Self {
        self.text_background_color = color;
        self
    }

    /// 设置最大缩放比例
    ///
    /// # Parameters
    /// - `max_scale`: 最大缩放比例，使用 `f32` 类型表示
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    pub fn max_scale(&mut self, max_scale: f32) -> &mut Self {
        self.max_scale = max_scale;
        self
    }

    /// 设置图片高度
    ///
    /// # Parameters
    /// - `pic_h`: 图片高度，必须是非零值
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    /// # Panics
    /// - 如果 `pic_h` 为零，程序将 panic
    ///
    pub fn pic_h(&mut self, pic_h: u32) -> &mut Self {
        assert_ne!(pic_h, 0, "Picture height must be non-zero.");
        self.pic_h = pic_h;
        self
    }

    /// 设置上部文本高度
    ///
    /// # Parameters
    /// - `text_up_h`: 上部文本高度，必须是非零值
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    /// # Panics
    /// - 如果 `text_up_h` 为零，程序将 panic
    ///
    pub fn text_up_h(&mut self, text_up_h: u32) -> &mut Self {
        assert_ne!(text_up_h, 0, "Upper text height must be non-zero.");
        self.text_up_h = text_up_h;
        self
    }

    /// 设置视频封面时间
    ///
    /// # Parameters
    /// - `video_cover_time`: 视频封面时间，使用 `u32` 类型表示
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    pub fn video_cover_time(&mut self, video_cover_time: u32) -> &mut Self {
        self.video_cover_time = video_cover_time;
        self
    }

    /// 设置视频结束时间
    ///
    /// # Parameters
    /// - `video_ending_time`: 视频结束时间，使用 `u32` 类型表示
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    pub fn video_ending_time(&mut self, video_ending_time: u32) -> &mut Self {
        self.video_ending_time = video_ending_time;
        self
    }

    /// 设置视频背景颜色
    ///
    /// # Parameters
    /// - `video_background_color`: 视频背景颜色，使用 `String` 类型表示
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    pub fn video_background_color(&mut self, video_background_color: String) -> &mut Self {
        self.video_background_color = video_background_color;
        self
    }

    /// 设置视频滑动速度
    ///
    /// # Parameters
    /// - `video_swip_speed`: 视频滑动速度，使用 `u32` 类型表示
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    pub fn video_swip_speed(&mut self, video_swip_speed: u32) -> &mut Self {
        self.video_swip_speed = video_swip_speed;
        self
    }

    /// 设置视频帧率
    ///
    /// # Parameters
    /// - `video_fps`: 视频帧率，使用 `u32` 类型表示
    ///
    /// # Results
    /// - 返回可变引用 `&mut Self`，以便链式调用。
    ///
    pub fn video_fps(&mut self, video_fps: u32) -> &mut Self {
        self.video_fps = video_fps;
        self
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_swipingimg_new_panic1() {
        let _t: BigImg = BigImg::new(&Path::new(""), &[]);
    }

    #[test]
    fn test_swipingimg_new_panic2() {
        let bind = &[Chunk::new(
            PathBuf::from("t1"),
            vec![String::from("value")],
            vec![String::from("456")],
        )
        .unwrap()];
        let t = BigImgBuilder::new(&Path::new(""), bind).pic_h(200).build();
        match t {
            Ok(_) => (),
            Err(e) => println!("{e:#?}"),
        }
    }
}
