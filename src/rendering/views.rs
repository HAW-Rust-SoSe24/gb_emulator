use macroquad::prelude::*;

pub struct GbDisplay {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scaling: f32,
}

impl GbDisplay {
    pub fn draw(&mut self, image: &Image) {
        let tex2d_params = DrawTextureParams {
            dest_size: Option::Some(Vec2::new(
                image.width() as f32 * self.scaling,
                image.height() as f32 * self.scaling,
            )),
            source: None,
            rotation: 0.,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };

        let tex2d = Texture2D::from_image(image);
        tex2d.set_filter(FilterMode::Nearest);
        draw_texture_ex(&tex2d, self.offset_x, self.offset_y, WHITE, tex2d_params);

        //TODO: Draw actual emulator content
        //draw_text(
        //    "Game Display",
        //    self.offset_x + 100.0,
        //    self.offset_y + 250.0,
        //    100.0,
        //    BLACK,
        //);
    }

    pub fn size(&self, image: &Image) -> Vec2 {
        Vec2 {
            x: image.width() as f32 * self.scaling,
            y: image.height() as f32 * self.scaling,
        }
    }
}

pub struct TileViewer {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scaling: f32,
}

impl TileViewer {
    pub fn draw(&mut self, atlas: &Image) {
        let tex2d_params = DrawTextureParams {
            dest_size: Option::Some(Vec2::new(
                atlas.width() as f32 * self.scaling,
                atlas.height() as f32 * self.scaling,
            )),
            source: None,
            rotation: 0.,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };

        let tex2d = Texture2D::from_image(atlas);
        tex2d.set_filter(FilterMode::Nearest);
        draw_texture_ex(&tex2d, self.offset_x, self.offset_y, WHITE, tex2d_params);

        let mouse_pos = mouse_position();

        if mouse_pos.0 >= self.offset_x
            && mouse_pos.0 < self.offset_x + 16.0 * 8.0 * self.scaling
            && mouse_pos.1 >= self.offset_y
            && mouse_pos.1 < self.offset_y + 24.0 * 8.0 * self.scaling
        {
            let x_tile = ((mouse_pos.0 - self.offset_x) / self.scaling).floor() as u32 / 8;
            let y_tile = ((mouse_pos.1 - self.offset_y) / self.scaling).floor() as u32 / 8;
            let tile_index = y_tile * 16 + x_tile;
            let tile_addr = 0x8000 + tile_index * 16;

            let pos_text = format!("Tile Index: {}, ADR: {:#X}", tile_index, tile_addr);

            draw_text(
                &pos_text,
                self.offset_x + 4.0,
                self.offset_y + 24.0 * 8.0 * self.scaling + 16.0,
                16.0,
                WHITE,
            );
        }
    }

    pub fn size(&self) -> Vec2 {
        Vec2 {
            x: 8.0 * 16.0 * self.scaling,
            y: 8.0 * 24.0 * self.scaling + 20.0,
        }
    }
}

pub struct BackgroundViewer {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scaling: f32,
}

impl BackgroundViewer {
    pub fn draw(&mut self, image: &Image) {
        let tex2d_params = DrawTextureParams {
            dest_size: Option::Some(Vec2::new(
                image.width() as f32 * self.scaling,
                image.height() as f32 * self.scaling,
            )),
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };

        let tex2d = Texture2D::from_image(image);
        tex2d.set_filter(FilterMode::Nearest);
        draw_texture_ex(&tex2d, self.offset_x, self.offset_y, WHITE, tex2d_params);
    }

    pub fn size(&self) -> Vec2 {
        Vec2 {
            x: (32.0 * 8.0) * self.scaling,
            y: (32.0 * 8.0) * self.scaling,
        }
    }
}

pub struct OnScreenControlLocations {
    pub a: Vec2,
    pub b: Vec2,
    pub select: Vec2,
    pub start: Vec2,
    pub cross_up: Vec2,
    pub cross_right: Vec2,
    pub cross_down: Vec2,
    pub cross_left: Vec2,
}

pub struct OnScreenControls {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scaling: f32,

    a: Texture2D,
    b: Texture2D,
    select: Texture2D,
    start: Texture2D,
    cross: Texture2D,
}

impl OnScreenControls {
    pub fn new(offset_x: f32, offset_y: f32, scaling: f32) -> OnScreenControls {
        let ec = OnScreenControls {
            offset_x,
            offset_y,
            scaling,
            a: Texture2D::from_image(
                &Image::from_file_with_format(
                    include_bytes!("../../assets/buttons/A-active.png"),
                    Some(ImageFormat::Png),
                )
                .expect("Asset not found"),
            ),
            b: Texture2D::from_image(
                &Image::from_file_with_format(
                    include_bytes!("../../assets/buttons/B-active.png"),
                    Some(ImageFormat::Png),
                )
                .expect("Asset not found"),
            ),
            select: Texture2D::from_image(
                &Image::from_file_with_format(
                    include_bytes!("../../assets/buttons/select-active.png"),
                    Some(ImageFormat::Png),
                )
                .expect("Asset not found"),
            ),
            start: Texture2D::from_image(
                &Image::from_file_with_format(
                    include_bytes!("../../assets/buttons/start-active.png"),
                    Some(ImageFormat::Png),
                )
                .expect("Asset not found"),
            ),
            cross: Texture2D::from_image(
                &Image::from_file_with_format(
                    include_bytes!("../../assets/buttons/cross.png"),
                    Some(ImageFormat::Png),
                )
                .expect("Asset not found"),
            ),
        };

        ec.a.set_filter(FilterMode::Nearest);
        ec.b.set_filter(FilterMode::Nearest);
        ec.select.set_filter(FilterMode::Nearest);
        ec.start.set_filter(FilterMode::Nearest);

        ec
    }

    pub fn draw(&self) {
        let ab_params = DrawTextureParams {
            dest_size: Option::Some(Vec2::new(
                self.a.width() * self.scaling,
                self.a.height() * self.scaling,
            )),
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };

        let select_start_params = DrawTextureParams {
            dest_size: Option::Some(Vec2::new(
                self.select.width() * self.scaling,
                self.select.height() * self.scaling,
            )),
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };

        let cross_params = DrawTextureParams {
            dest_size: Option::Some(Vec2::new(
                self.cross.width() * self.scaling,
                self.cross.height() * self.scaling,
            )),
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };

        draw_texture_ex(
            &self.a,
            self.offset_x + self.scaling * 520.0,
            self.offset_y + self.scaling * 50.0,
            WHITE,
            ab_params.clone(),
        );
        draw_texture_ex(
            &self.b,
            self.offset_x + self.scaling * 420.0,
            self.offset_y + self.scaling * 100.0,
            WHITE,
            ab_params,
        );
        draw_texture_ex(
            &self.select,
            self.offset_x + self.scaling * 240.0,
            self.offset_y + 220.0 * self.scaling,
            WHITE,
            select_start_params.clone(),
        );
        draw_texture_ex(
            &self.start,
            self.offset_x + self.scaling * (self.select.width() + 260.0),
            self.offset_y + 220.0 * self.scaling,
            WHITE,
            select_start_params,
        );
        draw_texture_ex(
            &self.cross,
            self.offset_x,
            self.offset_y + self.scaling * 30.0,
            WHITE,
            cross_params,
        );
    }

    pub fn get_on_screen_control_locations(&self) -> OnScreenControlLocations {
        OnScreenControlLocations {
            a: Vec2::new(
                self.offset_x + self.scaling * (520.0 + 40.0),
                self.offset_y + self.scaling * (50.0 + 40.0),
            ),
            b: Vec2::new(
                self.offset_x + self.scaling * (420.0 + 40.0),
                self.offset_y + self.scaling * (100.0 + 40.0),
            ),
            select: Vec2::new(
                self.offset_x + self.scaling * (240.0 + 40.0),
                self.offset_y + self.scaling * (220.0 + 15.0),
            ),
            start: Vec2::new(
                self.offset_x + self.scaling * (self.select.width() + 260.0 + 40.0),
                self.offset_y + self.scaling * (220.0 + 15.0),
            ),
            cross_up: Vec2::new(self.offset_x + self.scaling * 110.0, self.offset_y + self.scaling * 70.0),
            cross_right: Vec2::new(self.offset_x + self.scaling * 180.0, self.offset_y + self.scaling * 142.0),
            cross_down: Vec2::new(self.offset_x + self.scaling * 110.0, self.offset_y + self.scaling * 215.0),
            cross_left: Vec2::new(self.offset_x + self.scaling * 40.0, self.offset_y + self.scaling * 142.0),
        }
    }
}
