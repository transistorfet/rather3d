use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::f64::consts::PI;

use nalgebra::{Point3, Vector3, Vector4, Matrix4, Perspective3};
use piston_window::{PistonWindow, WindowSettings, clear, Line, DrawState, EventLoop, Events, EventSettings, RenderEvent, Button, Key, PressEvent};


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
                        let point: Point3<f64> = Vector3::from_iterator(words
                            .map(|w| str::parse::<f64>(w).unwrap())).into();
                        points.push(point);
                    },
                    "f" => {
                        let face: Vec<usize> = words
                            .map(|w| str::parse::<usize>(w).unwrap())
                            .collect();
                        faces.push(face);
                    },
                    _ => { },
                }
            }
        }

        Ok(Object {
            points,
            faces,
        })
    }

    pub fn project(&self, camera: Vector3<f64>, rotation: f64) -> Vec<Point3<f64>> {
        let scale = Self::scale(1.0);
        let rotate_z = Self::rotate_z(180.0);
        let rotate_y = Self::rotate_y(rotation);
        //let translate = Self::translate(800.0, 800.0, -1000.0);
        let translate = Self::translate(0.0, 0.0, camera[2]);

        //let perspective_from_camera = Self::perspective_transform_fov(PI / 4.0, 1.0, 0.1, 5000.0);
        let perspective_from_camera = Self::perspective_transform_fov(PI / 4.0, SIZE_X / SIZE_Y, 1.0, 10000.0);
        //let perspective_from_camera = Perspective3::new(16.0 / 9.0, 3.14 / 4.0, 1.0, 10000.0).to_homogeneous();
        //let perspective_from_camera = Perspective3::new(SIZE_X / SIZE_Y, 3.14 / 4.0, 1.0, 10000.0).to_homogeneous();

        self.points
            .iter()
            .map(|point| point.to_homogeneous())
            .map(|point| perspective_from_camera * translate * scale * rotate_y * rotate_z * point)
            .map(|point| Point3::from_homogeneous(point).unwrap())

            //.map(|point| translate * scale * rotate_y * rotate_z * point.to_homogeneous())
            //.map(|point| Point3::new(point[0], point[1], point[2]))
            //.map(|point| perspective_from_camera.project_point(&point))

            .collect()
    }

    pub fn scale(scale: f64) -> Matrix4<f64> {
        Matrix4::new(
            scale,   0.0,   0.0, 0.0,
              0.0, scale,   0.0, 0.0,
              0.0,   0.0, scale, 0.0,
              0.0,   0.0,   0.0, 1.0,
        )
    }

    pub fn translate(x: f64, y: f64, z: f64) -> Matrix4<f64> {
        Matrix4::new(
              1.0,   0.0,   0.0,   x,
              0.0,   1.0,   0.0,   y,
              0.0,   0.0,   1.0,   z,
              0.0,   0.0,   0.0, 1.0,
        )
    }

    pub fn rotate_x(x: f64) -> Matrix4<f64> {
        let x = x * PI / 180.0;
        Matrix4::new(
            1.0,        0.0,     0.0, 0.0,
            0.0,    x.cos(), x.sin(), 0.0,
            0.0, -(x.sin()), x.cos(), 0.0,
            0.0,        0.0,     0.0, 1.0,
        )
    }

    pub fn rotate_y(y: f64) -> Matrix4<f64> {
        let y = y * PI / 180.0;
        Matrix4::new(
               y.cos(),     0.0, y.sin(), 0.0,
                   0.0,     1.0,     0.0, 0.0,
            -(y.sin()),     0.0, y.cos(), 0.0,
                   0.0,     0.0,     0.0, 1.0,
        )
    }

    pub fn rotate_z(z: f64) -> Matrix4<f64> {
        let z = z * PI / 180.0;
        Matrix4::new(
               z.cos(), z.sin(), 0.0, 0.0,
            -(z.sin()), z.cos(), 0.0, 0.0,
                   0.0,     0.0, 1.0, 0.0,
                   0.0,     0.0, 0.0, 1.0,
        )
    }

    pub fn perspective_transform_fov(fov: f64, aspect: f64, n: f64, f: f64) -> Matrix4<f64> {
        let e = 1.0 / (fov / 2.0).tan();
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

fn get_point(points: &[Point3<f64>], face: usize) -> [f64; 2] {
    [
        ((points[face - 1][0] + 1.0) / 2.0) * SIZE_X,
        ((points[face - 1][1] + 1.0) / 2.0) * SIZE_Y
    ]
}

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [SIZE_X, SIZE_Y])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let object = Object::read("data/cessna.obj").unwrap();
    //let object = Object::read("data/diamond.obj").unwrap();

    let mut z = -100.0;
    let mut rotation = 0.0;

    let mut events = Events::new(EventSettings::new().lazy(true));
    while let Some(e) = events.next(&mut window) {

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Up => { z += 1.0; }
                Key::Down => { z -= 1.0; }
                _ => {},
            }
        }

        if let Some(args) = e.render_args() {
            window.draw_2d(&e, |c, g, _device| {
                println!("start drawing");

                clear([1.0; 4], g);
                //rectangle(BLUE,
                //          [0.0, 0.0, 100.0, 100.0],
                //          c.transform, g);

                let points = object.project(Vector3::new(0.0, 20.0, z), rotation);
                rotation += 4.0;

                //println!("{:?}", points);

                //Line::new(BLUE, 0.4)
                //    .draw_from_to([0.0, 100.0], [100.0, 100.0], &c.draw_state, c.transform, g);
                //Line::new(BLUE, 0.4)
                //    .draw_from_to([100.0, 100.0], [100.0, 0.0], &c.draw_state, c.transform, g);

                for face in &object.faces {
                    let p1 = get_point(&points, face[0]);
                    let p2 = get_point(&points, face[1]);
                    let p3 = get_point(&points, face[2]);

                    //println!("{:?} {:?} {:?}", p1, p2, p3);

                    Line::new(BLUE, 0.2)
                        .draw_from_to(p1, p2, &c.draw_state, c.transform, g);
                    Line::new(BLUE, 0.2)
                        .draw_from_to(p2, p3, &c.draw_state, c.transform, g);
                    Line::new(BLUE, 0.2)
                        .draw_from_to(p3, p1, &c.draw_state, c.transform, g);
                }
            });
        }
    }
}
