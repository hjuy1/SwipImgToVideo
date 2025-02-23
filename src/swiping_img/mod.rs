pub mod chunk;
mod draw;

use crate::{
    err_new, err_new_image, err_new_io, err_new_tryfrom,
    error::{Kind, Result},
    prelude::debug_print,
};
use ab_glyph::FontVec;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use std::{
    fmt::{self, Debug},
    path::{Path, PathBuf},
    process::Command,
};
pub use {chunk::Chunk, draw::Draw};

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
    /// 组合所有图像块并生成最终视频。
    ///
    /// # Parameters
    /// - `save_name`: 最终视频文件名。
    ///
    /// # Errors
    /// - 如果图像处理或保存过程中发生错误，则返回 `Err`。
    /// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
    ///
    pub fn run<P: AsRef<Path>>(&self, save_name: P) -> Result<()> {
        let chunks = self.divide();
        let mut results = Vec::with_capacity(chunks.len() + 2);

        for (index, &chunk) in chunks.iter().enumerate() {
            let target = self.combain_chunk(chunk)?;
            if index == 0 {
                let cover = target.crop_imm(0, 0, self.screen.0, self.screen.1);
                let cover_pic_name = Path::new("cover.png");
                // 保存组合后的图像
                cover
                    .save(self.work_dir.join(cover_pic_name))
                    .map_err(|e| err_new_image!(e))?;
                debug_print(format!("{cover_pic_name:?} successed"));

                let cover_video_name = cover_pic_name.with_extension("mp4");
                self.generate_endpoint_video(
                    cover_pic_name,
                    &cover_video_name,
                    self.video_cover_time,
                )?;
                results.push(cover_video_name);
            }

            // 保存组合后的图像
            let mid_pic_name = format!("{index:0>2}.png");
            let mid_pic_name = Path::new(&mid_pic_name);
            target
                .save(self.work_dir.join(mid_pic_name))
                .map_err(|e| err_new_image!(e))?;
            debug_print(format!("{mid_pic_name:?} successed"));

            let mid_video_name = mid_pic_name.with_extension("mp4");
            self.generate_mid_video(chunk.len() as u32, mid_pic_name, &mid_video_name)?;
            results.push(mid_video_name);

            if index == chunks.len() - 1 {
                let w = target.dimensions().0;
                let ending = target.crop_imm(w - self.screen.0, 0, self.screen.0, self.screen.1);
                let ending_pic_name = Path::new("ending.png");
                // 保存组合后的图像
                ending
                    .save(self.work_dir.join(ending_pic_name))
                    .map_err(|e| err_new_image!(e))?;
                debug_print(format!("{ending_pic_name:?} successed"));

                let ending_video_name = ending_pic_name.with_extension("mp4");
                self.generate_endpoint_video(
                    ending_pic_name,
                    &ending_video_name,
                    self.video_ending_time,
                )?;
                results.push(ending_video_name);
            }
        }

        self.combain(&mut results, save_name.as_ref())?;
        Ok(())
    }

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

    /// 将多个图像块组合成一个完整的图像。
    ///
    /// # Parameters
    /// - `chunk`: 要组合的图像块切片。
    ///
    /// # Results
    /// 如果成功，则返回组合后的 `DynamicImage`；如果失败，则返回 `Err`。
    ///
    /// # Errors
    /// - 如果 `chunk` 为空，则返回 `Err`。
    /// - 如果图像处理过程中发生错误，则返回 `Err`。
    ///
    fn combain_chunk(&self, chunk: &[Chunk]) -> Result<DynamicImage> {
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
        Ok(target)
    }

    /// 生成视频封面或结尾视频。
    ///
    /// # Parameters
    /// - `pic_name`: 素材图片名称。
    /// - `video_name`: 生成视频名称。
    /// - `video_time`: 视频时长（秒）。
    ///
    /// # Errors
    /// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
    ///
    fn generate_endpoint_video(
        &self,
        pic_name: &Path,
        video_name: &Path,
        video_time: u32,
    ) -> Result<()> {
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
        debug_print(format!("{video_name:?} successed"));
        Ok(())
    }

    /// 生成中间部分的视频。
    ///
    /// # Parameters
    /// - `len`: 素材图片中 `chunk` 数量。
    /// - `pic_name`: 素材图片名称。
    /// - `video_name`: 生成视频名称。
    ///
    /// # Errors
    /// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
    ///
    fn generate_mid_video(&self, len: u32, pic_name: &Path, video_name: &Path) -> Result<()> {
        let adjust_len = len - self.overlap;
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
        debug_print(format!("{video_name:?} successed"));
        Ok(())
    }

    #[allow(unused)]
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

    /// 合并多个文件为单个输出文件，使用ffmpeg的concat协议
    ///
    /// # Parameters
    /// - `results`: 需要合并的源文件路径列表
    /// - `save_name`: 合并后的输出文件路径
    ///
    /// # Errors
    /// - 如果文件写入或 `FFmpeg` 命令执行失败，则返回 `Err`。
    ///
    fn combain(&self, results: &mut [PathBuf], save_name: &Path) -> Result<()> {
        // 构建ffmpeg concat协议要求的输入文件列表字符串
        // 格式示例：file '/path/to/file1'\nfile '/path/to/file2'
        let result_str =
            results
                .iter()
                .fold(String::with_capacity(results.len() * 20), |mut init, s| {
                    init.push_str(&format!("file {}\n", s.to_string_lossy()));
                    init
                });

        // 将文件列表写入临时文本文件
        let list_file = self.work_dir.join("list.txt");
        std::fs::write(&list_file, result_str)?;

        // 调用ffmpeg执行合并操作参数说明：
        // -f concat 指定concat分离器
        // -i 输入文件列表
        // -c copy 使用流拷贝模式（不重新编码）
        // -y 覆盖输出文件
        self.ffmpeg(&[
            "-f",
            "concat",
            "-i",
            &list_file.to_string_lossy(),
            "-c",
            "copy",
            "-y",
            &save_name.to_string_lossy(),
        ])?;

        println!("{} successed", save_name.to_string_lossy());

        // 清理临时文件（包含两个步骤）：
        // 1. 删除文件列表
        // 2. 删除所有中间结果文件及其对应的png文件
        let _ = std::fs::remove_file(&list_file);
        for result in results {
            let _ = std::fs::remove_file(self.work_dir.join(&result));
            result.set_extension("png");
            let _ = std::fs::remove_file(self.work_dir.join(result));
        }
        println!("cleanup successed");
        Ok(())
    }
}

impl Debug for BigImg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BigImg")
            .field("work_dir", &self.work_dir)
            .field("len_chunks", &self.chunks.len())
            .field("chunk1", &self.chunks[0])
            .field("screen", &self.screen)
            .field("step", &self.step)
            .field("width_chunk", &self.width_chunk)
            .field("overlap", &self.overlap)
            .field("text_background_color", &self.text_background_color)
            .field("text_color", &self.text_color)
            .field("max_scale", &self.max_scale)
            .field("pic_h", &self.pic_h)
            .field("text_up_h", &self.text_up_h)
            .field("text_down_h", &self.text_down_h)
            .field("font", &self.font)
            .field("video_cover_time", &self.video_cover_time)
            .field("video_ending_time", &self.video_ending_time)
            .field("video_background_color", &self.video_background_color)
            .field("video_swip_speed", &self.video_swip_speed)
            .field("video_fps", &self.video_fps)
            .finish()
    }
}

pub struct BigImgBuilder<'a> {
    work_dir: PathBuf,
    chunks: &'a [Chunk],
    screen: (u32, u32),
    step: u32,
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
            step: 40,
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
        if !self.work_dir.exists() {
            return Err(err_new!(Kind::BigImgBuilderError, "work_dir is not exist"));
        }
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
    pub fn text_color(&mut self, text_color: Rgba<u8>) -> &mut Self {
        self.text_color = text_color;
        self
    }

    /// 设置文本背景颜色
    ///
    /// # Parameters
    /// - `color`: 文本背景颜色，使用 `(Rgba<u8>, Rgba<u8>)` 类型表示
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
    pub fn max_scale(&mut self, max_scale: f32) -> &mut Self {
        self.max_scale = max_scale;
        self
    }

    /// 设置图片高度
    ///
    /// # Parameters
    /// - `pic_h`: 图片高度，必须是非零值
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
    pub fn video_cover_time(&mut self, video_cover_time: u32) -> &mut Self {
        self.video_cover_time = video_cover_time;
        self
    }

    /// 设置视频结束时间
    ///
    /// # Parameters
    /// - `video_ending_time`: 视频结束时间，使用 `u32` 类型表示
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
    pub fn video_background_color(&mut self, video_background_color: String) -> &mut Self {
        self.video_background_color = video_background_color;
        self
    }

    /// 设置视频滑动速度
    ///
    /// # Parameters
    /// - `video_swip_speed`: 视频滑动速度，使用 `u32` 类型表示
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
    pub fn video_fps(&mut self, video_fps: u32) -> &mut Self {
        self.video_fps = video_fps;
        self
    }
}
