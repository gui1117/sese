use ::graphics::Vertex;

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
    Vertex { position: [-1.0, -1.0, 0.05], tex_coords: [0.5, 0.5] },
    Vertex { position: [-1.0, 1.0, -0.05], tex_coords: [0.6, 0.5] },

    Vertex { position: [-1.0, -1.0, 0.05], tex_coords: [0.5, 0.5] },
    Vertex { position: [-1.0, 1.0, 0.05], tex_coords: [0.6, 0.5] },
    Vertex { position: [-1.0, 1.0, -0.05], tex_coords: [0.6, 0.0] },

    // x=[0.6, 0.7], y=[0.0, 0.5]
    Vertex { position: [1.0, -1.0, 0.05], tex_coords: [0.6, 0.5] },
    Vertex { position: [1.0, -1.0, -0.05], tex_coords: [0.6, 0.0] },
    Vertex { position: [1.0, 1.0, -0.05], tex_coords: [0.7, 0.0] },

    Vertex { position: [1.0, 1.0, 0.05], tex_coords: [0.7, 0.5] },
    Vertex { position: [1.0, -1.0, 0.05], tex_coords: [0.6, 0.5] },
    Vertex { position: [1.0, 1.0, -0.05], tex_coords: [0.7, 0.0] },

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
                vertex.tex_coords[0] = 0.5 + 0.1/width as f32;
            }
            if vertex.tex_coords[0] == 0.7 {
                vertex.tex_coords[0] = 0.5 + 0.2/width as f32;
            }
            if vertex.tex_coords[1] == 0.6 {
                vertex.tex_coords[1] = 0.5 + 0.1/height as f32;
            }
            if vertex.tex_coords[1] == 0.7 {
                vertex.tex_coords[1] = 0.5 + 0.2/height as f32;
            }
            // IDEA: unaligned tiles can be cool
            vertex.position[0] *= width as f32/2.0;
            vertex.position[1] *= height as f32/2.0;

            vertex.position[0] = vertex.position[0] - vertex.position[0].signum()*0.05;
            vertex.position[1] = vertex.position[1] - vertex.position[1].signum()*0.05;

            vertex
        })
        .collect::<Vec<_>>()
}
