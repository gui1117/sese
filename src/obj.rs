use ::graphics::Vertex;
use std::f32::consts::PI;

const THICKNESS: f32 = 0.05;
const CYLINDER_DIV: usize = 32;

const TILE: [Vertex; 36] = [
    // Bottom
    Vertex { position: [1.0, -1.0, -0.05], tex_coords: [0.5, 0.0] },
    Vertex { position: [-1.0, -1.0, -0.05], tex_coords: [0.0, 0.0] },
    Vertex { position: [-1.0, 1.0, -0.05], tex_coords: [0.0, 0.5] },

    Vertex { position: [1.0, 1.0, -0.05], tex_coords: [0.5, 0.5] },
    Vertex { position: [1.0, -1.0, -0.05], tex_coords: [0.5, 0.0] },
    Vertex { position: [-1.0, 1.0, -0.05], tex_coords: [0.0, 0.5] },

    // Top
    Vertex { position: [-1.0, -1.0, 0.05], tex_coords: [0.5, 0.5] },
    Vertex { position: [1.0, -1.0, 0.05], tex_coords: [1.0, 0.5] },
    Vertex { position: [-1.0, 1.0, 0.05], tex_coords: [0.5, 1.0] },

    Vertex { position: [1.0, -1.0, 0.05], tex_coords: [1.0, 0.5] },
    Vertex { position: [1.0, 1.0, 0.05], tex_coords: [1.0, 1.0] },
    Vertex { position: [-1.0, 1.0, 0.05], tex_coords: [0.5, 1.0] },

    // Sides
    // x=[0.5, 0.6], y=[0.0, 0.5]
    Vertex { position: [-1.0, -1.0, -0.05], tex_coords: [0.5, 0.0] },
    Vertex { position: [-1.0, -1.0, 0.05], tex_coords: [0.6, 0.0] },
    Vertex { position: [-1.0, 1.0, -0.05], tex_coords: [0.5, 0.5] },

    Vertex { position: [-1.0, -1.0, 0.05], tex_coords: [0.6, 0.0] },
    Vertex { position: [-1.0, 1.0, 0.05], tex_coords: [0.6, 0.5] },
    Vertex { position: [-1.0, 1.0, -0.05], tex_coords: [0.5, 0.5] },

    // x=[0.6, 0.7], y=[0.0, 0.5]
    Vertex { position: [1.0, -1.0, 0.05], tex_coords: [0.7, 0.0] },
    Vertex { position: [1.0, -1.0, -0.05], tex_coords: [0.6, 0.0] },
    Vertex { position: [1.0, 1.0, -0.05], tex_coords: [0.6, 0.5] },

    Vertex { position: [1.0, 1.0, 0.05], tex_coords: [0.7, 0.5] },
    Vertex { position: [1.0, -1.0, 0.05], tex_coords: [0.7, 0.0] },
    Vertex { position: [1.0, 1.0, -0.05], tex_coords: [0.6, 0.5] },

    // x=[0.5, 1.0], y=[0.5, 0.6]
    Vertex { position: [-1.0, -1.0, -0.05], tex_coords: [0.5, 0.5] },
    Vertex { position: [1.0, -1.0, -0.05], tex_coords: [1.0, 0.5] },
    Vertex { position: [-1.0, -1.0, 0.05], tex_coords: [0.5, 0.6] },

    Vertex { position: [1.0, -1.0, 0.05], tex_coords: [1.0, 0.6] },
    Vertex { position: [-1.0, -1.0, 0.05], tex_coords: [0.5, 0.6] },
    Vertex { position: [1.0, -1.0, -0.05], tex_coords: [1.0, 0.5] },

    // x=[0.5, 1.0], y=[0.6, 0.7]
    Vertex { position: [1.0, 1.0, -0.05], tex_coords: [1.0, 0.6] },
    Vertex { position: [-1.0, 1.0, -0.05], tex_coords: [0.5, 0.6] },
    Vertex { position: [-1.0, 1.0, 0.05], tex_coords: [0.5, 0.7] },

    Vertex { position: [-1.0, 1.0, 0.05], tex_coords: [0.5, 0.7] },
    Vertex { position: [1.0, 1.0, 0.05], tex_coords: [1.0, 0.7] },
    Vertex { position: [1.0, 1.0, -0.05], tex_coords: [1.0, 0.6] },
];

pub fn generate_tile(width: isize, height: isize) -> Vec<Vertex> {
    TILE.iter()
        .cloned()
        .map(|mut vertex| {
            if vertex.tex_coords[0] == 0.6 {
                vertex.tex_coords[0] = 0.5 + THICKNESS/width as f32;
            }
            if vertex.tex_coords[0] == 0.7 {
                vertex.tex_coords[0] = 0.5 + THICKNESS*2.0/width as f32;
            }
            if vertex.tex_coords[1] == 0.6 {
                vertex.tex_coords[1] = 0.5 + THICKNESS/height as f32;
            }
            if vertex.tex_coords[1] == 0.7 {
                vertex.tex_coords[1] = 0.5 + THICKNESS*2.0/height as f32;
            }
            // IDEA: unaligned tiles can be cool
            vertex.position[0] *= width as f32/2.0;
            vertex.position[1] *= height as f32/2.0;

            vertex.position[0] = vertex.position[0] - vertex.position[0].signum()*0.05;
            vertex.position[1] = vertex.position[1] - vertex.position[1].signum()*0.05;

            vertex.position[2] = vertex.position[2].signum()*THICKNESS/2.0;

            vertex
        })
        .collect::<Vec<_>>()
}

pub fn generate_tube(size: isize) -> Vec<Vertex> {
    let mut vertices = vec![];
    for i in 0..CYLINDER_DIV {
        let div0 = (i as f32) / CYLINDER_DIV as f32;
        let div1 = ((i + 1) as f32) / CYLINDER_DIV as f32;

        let a0 = div0 * 2.0 * PI;
        let a1 = div1 * 2.0 * PI;

        let mut p0 = [a0.cos(), a0.sin()];
        let mut p1 = [a1.cos(), a1.sin()];
        for p in p0.iter_mut().chain(&mut p1.iter_mut()) {
            *p = *p*::CFG.column_outer_radius;
        }

        let y = size as f32/2.0 - THICKNESS/2.0;

        vertices.push(Vertex { position: [p0[0], -y, p0[1]], tex_coords: [p0[0]+0.5, p0[1]+0.5] });
        vertices.push(Vertex { position: [p1[0], -y, p1[1]], tex_coords: [p1[0]+0.5, p1[1]+0.5] });
        vertices.push(Vertex { position: [0.0, -y, 0.0], tex_coords: [0.5, 0.5] });

        vertices.push(Vertex { position: [p1[0], y, p1[1]], tex_coords: [p1[0]+0.5, p1[1]+0.5] });
        vertices.push(Vertex { position: [p0[0], y, p0[1]], tex_coords: [p0[0]+0.5, p0[1]+0.5] });
        vertices.push(Vertex { position: [0.0, y, 0.0], tex_coords: [0.5, 0.5] });

        vertices.push(Vertex { position: [p0[0], -y, p0[1]], tex_coords: [div0, 0.0] });
        vertices.push(Vertex { position: [p0[0], y, p0[1]], tex_coords: [div0, y] });
        vertices.push(Vertex { position: [p1[0], y, p1[1]], tex_coords: [div1, y] });

        vertices.push(Vertex { position: [p1[0], -y, p1[1]], tex_coords: [div1, 0.0] });
        vertices.push(Vertex { position: [p0[0], -y, p0[1]], tex_coords: [div0, 0.0] });
        vertices.push(Vertex { position: [p1[0], y, p1[1]], tex_coords: [div1, y] });

    }

    vertices
}
