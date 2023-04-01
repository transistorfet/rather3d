use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::f64::consts::PI;

use nalgebra::{Point3, Vector3, Vector4, Matrix4, Perspective3};
use piston_window::{
    PistonWindow, WindowSettings, clear, Line, Text, DrawState, EventLoop, Events, EventSettings, RenderEvent, Button, Key,
    PressEvent, ReleaseEvent, MouseRelativeEvent, ResizeEvent, IdleEvent, TextureSettings,
};
use opengl_graphics::{GlGraphics, OpenGL, GlyphCache};

#[derive(Clone, Debug)]
struct Object {
    points: Vec<Point3<f64>>,
    faces: Vec<Vec<usize>>,
}

impl Object {
    pub fn read(filename: &str) -> Result<Object, io::Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let mut points = vec![];
        let mut faces = vec![];

        for line in reader.lines() {
            let line = line?;
            let mut words = line.split_whitespace();
            if let Some(line_type) = words.next() {
                match line_type {
                    "v" => {
                        let point: Point3<f64> = Vector3::from_iterator(words.map(|w| str::parse::<f64>(w).unwrap())).into();
                        points.push(point);
                    },
                    "f" => {
                        let face: Vec<usize> = words.map(|w| str::parse::<usize>(w).unwrap()).collect();
                        faces.push(face);
                    },
                    _ => {},
                }
            }
        }

        Ok(Object {
            points,
            faces,
        })
    }

    pub fn project(
        &self,
        camera_position: Point3<f64>,
        camera_orientation: Vector3<f64>,
        window_size: [f64; 2],
    ) -> Vec<Point3<f64>> {
        let object_position = Point3::new(0.0, 0.0, 100.0);

        let scale = Self::scale(1.0);
        let rotate_z = Self::rotate_z(0.0);
        let rotate_y = Self::rotate_y(0.0);
        //let translate = Self::translate(800.0, 800.0, -1000.0);
        let translate = Self::translate(object_position);
        let world_from_object = translate * scale * rotate_y * rotate_z;

        //let perspective_from_camera = Self::perspective_transform_fov(PI / 4.0, 1.0, 0.1, 5000.0);
        let perspective_from_camera = Self::perspective_transform_fov(PI / 4.0, window_size[0] / window_size[1], 1.0, 10000.0);
        //let perspective_from_camera = Perspective3::new(16.0 / 9.0, 3.14 / 4.0, 1.0, 10000.0).to_homogeneous();
        //let perspective_from_camera = Perspective3::new(window_size[0] / window_size[1], 3.14 / 4.0, 1.0, 10000.0).to_homogeneous();

        let camera_from_world = Self::rotate(camera_orientation) * Self::translate(-1.0 * camera_position);

        self.points
            .iter()
            .map(|point| point.to_homogeneous())
            .map(|point| perspective_from_camera * camera_from_world * world_from_object * point)
            .map(|point| Point3::from_homogeneous(point).unwrap())
            //.map(|point| translate * scale * rotate_y * rotate_z * point.to_homogeneous())
            //.map(|point| Point3::new(point[0], point[1], point[2]))
            //.map(|point| perspective_from_camera.project_point(&point))
            .collect()
    }

    pub fn scale(scale: f64) -> Matrix4<f64> {
        #[rustfmt::skip]
        Matrix4::new(
            scale,   0.0,   0.0, 0.0,
              0.0, scale,   0.0, 0.0,
              0.0,   0.0, scale, 0.0,
              0.0,   0.0,   0.0, 1.0,
        )
    }

    pub fn translate(point: Point3<f64>) -> Matrix4<f64> {
        #[rustfmt::skip]
        Matrix4::new(
              1.0,   0.0,   0.0,   point[0],
              0.0,   1.0,   0.0,   point[1],
              0.0,   0.0,   1.0,   point[2],
              0.0,   0.0,   0.0, 1.0,
        )
    }

    pub fn rotate(vector: Vector3<f64>) -> Matrix4<f64> {
        Self::rotate_x(vector[0]) * Self::rotate_y(vector[1]) * Self::rotate_z(vector[2])
    }

    pub fn rotate_x(x: f64) -> Matrix4<f64> {
        let x = x * PI / 180.0;
        #[rustfmt::skip]
        Matrix4::new(
            1.0,        0.0,     0.0, 0.0,
            0.0,    x.cos(), x.sin(), 0.0,
            0.0, -(x.sin()), x.cos(), 0.0,
            0.0,        0.0,     0.0, 1.0,
        )
    }

    pub fn rotate_y(y: f64) -> Matrix4<f64> {
        let y = y * PI / 180.0;
        #[rustfmt::skip]
        Matrix4::new(
               y.cos(),     0.0, y.sin(), 0.0,
                   0.0,     1.0,     0.0, 0.0,
            -(y.sin()),     0.0, y.cos(), 0.0,
                   0.0,     0.0,     0.0, 1.0,
        )
    }

    pub fn rotate_z(z: f64) -> Matrix4<f64> {
        let z = z * PI / 180.0;
        #[rustfmt::skip]
        Matrix4::new(
               z.cos(), z.sin(), 0.0, 0.0,
            -(z.sin()), z.cos(), 0.0, 0.0,
                   0.0,     0.0, 1.0, 0.0,
                   0.0,     0.0, 0.0, 1.0,
        )
    }

    pub fn perspective_transform_fov(fov: f64, aspect: f64, n: f64, f: f64) -> Matrix4<f64> {
        let e = 1.0 / (fov / 2.0).tan();
        #[rustfmt::skip]
        Matrix4::new(
          e / aspect,   0.0,                 0.0,                       0.0,
                 0.0,   e,                   0.0,                       0.0,
                 0.0,   0.0,   (f + n) / (n - f),   (2.0 * f * n) / (n - f),
                 0.0,   0.0,                -1.0,                       0.0,
        )
    }
}

const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

const SIZE_X: f64 = 1920.0;
const SIZE_Y: f64 = 1080.0;

fn get_point(points: &[Point3<f64>], face: usize, window_size: [f64; 2]) -> ([f64; 2], bool) {
    (
        [
            ((points[face - 1][0] + 1.0) / 2.0) * window_size[0],
            ((points[face - 1][1] + 1.0) / 2.0) * window_size[1],
        ],
        points[face - 1][2] < 1.0,
    )
}

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [SIZE_X, SIZE_Y])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let ref mut gl = GlGraphics::new(opengl);

    let object = Object::read("data/cessna.obj").unwrap();
    //let object = Object::read("data/diamond.obj").unwrap();

    let font = "/usr/share/fonts/truetype/agave/agave-r-autohinted.ttf";
    let mut glyphs = GlyphCache::new(font, (), TextureSettings::new()).unwrap();

    let mut window_size = [SIZE_X, SIZE_Y];
    let mut forward = 0.0;
    let mut dry = 0.0;
    let mut cursor = [0.0; 2];
    let mut camera_position = Point3::new(0.0_f64, 0.0, 0.0);
    let mut camera_orientation = Vector3::new(0.0_f64, 0.0, 0.0);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        e.resize(|args| {
            window_size = args.window_size;
        });

        e.mouse_relative(|pos| {
            cursor = pos;
            camera_orientation.y += pos[0];
            camera_orientation.x += pos[1];
        });

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Up => {
                    forward = -1.0;
                },
                Key::Down => {
                    forward = 1.0;
                },
                Key::Left => {
                    dry = 1.0;
                },
                Key::Right => {
                    dry = -1.0;
                },
                _ => {},
            }
        }

        if let Some(button) = e.release_args() {
            match button {
                Button::Keyboard(Key::Up) | Button::Keyboard(Key::Down) => {
                    forward = 0.0;
                },
                Button::Keyboard(Key::Left) | Button::Keyboard(Key::Right) => {
                    dry = 0.0;
                },
                _ => {},
            }
        }

        camera_orientation.y -= dry;
        camera_position.x += forward * camera_orientation.y.to_radians().sin();
        camera_position.z -= forward * camera_orientation.y.to_radians().cos();
        //println!("position: {:?}, orientation: {:?}", camera_position, camera_orientation);

        if let Some(args) = e.idle_args() {}

        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                println!("start drawing");

                Text::new_color(BLUE, 12)
                    .draw_pos(
                        &format!("mouse: {:?} {:?}", cursor[0], cursor[1]),
                        [0.0, 24.0].into(),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform,
                        g,
                    )
                    .unwrap();

                Text::new_color(BLUE, 12)
                    .draw_pos(
                        &format!("position: {:?}, orientation: {:?}", camera_position, camera_orientation),
                        [0.0, 12.0].into(),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform,
                        g,
                    )
                    .unwrap();

                clear([1.0; 4], g);
                //rectangle(BLUE,
                //          [0.0, 0.0, 100.0, 100.0],
                //          c.transform, g);

                let points = object.project(camera_position, camera_orientation, window_size);
                //rotation += 4.0;

                //println!("{:?}", points);

                //Line::new(BLUE, 0.4)
                //    .draw_from_to([0.0, 100.0], [100.0, 100.0], &c.draw_state, c.transform, g);
                //Line::new(BLUE, 0.4)
                //    .draw_from_to([100.0, 100.0], [100.0, 0.0], &c.draw_state, c.transform, g);

                for face in &object.faces {
                    let (p1, p1_clipped) = get_point(&points, face[0], window_size);
                    let (p2, p2_clipped) = get_point(&points, face[1], window_size);
                    let (p3, p3_clipped) = get_point(&points, face[2], window_size);

                    if p1_clipped && p2_clipped && p3_clipped {
                        continue;
                    }

                    //println!("{:?} {:?} {:?}", p1, p2, p3);

                    Line::new(BLUE, 0.2).draw_from_to(p1, p2, &c.draw_state, c.transform, g);
                    Line::new(BLUE, 0.2).draw_from_to(p2, p3, &c.draw_state, c.transform, g);
                    Line::new(BLUE, 0.2).draw_from_to(p3, p1, &c.draw_state, c.transform, g);
                }
            });
        }
    }
}
