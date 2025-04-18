struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Mirror of `Uniforms` from `./mod.rs`. 
struct Uniforms {
    color_scheme: ColorScheme,

    // Really, we just need 91 bytes for this. However, wgsl needs
    // to have uniforms be 16-byte aligned and 16-byte stride. 92 `u32`s
    // would align but wouldn't have correct stride. So we need to use
    // `vec4u`s for the stride
    tile_flags: array<vec4u, 23>,
};

struct ColorScheme {
    tiles: array<vec4f, 3>,
    background: vec4f,
    selected: vec4f,
    highlighted: vec4f,
}

const SELECTED: u32 = 1 << 0;
const HIGHLIGHTED: u32 = 1 << 1;

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    var out: vec4f;

    let center_grid = nearest_center(in.uv, vec2f(0, 0), STEP_SIZE);
    let center_offset = nearest_center(in.uv, OFFSET, STEP_SIZE);
    let center = select(center_grid, center_offset, distance(in.uv, center_grid) > distance(in.uv, center_offset));

    let position = hexagonal_position(center, OFFSET);
    if !is_valid_position(position) {
        return uniforms.color_scheme.background;
    }

    let index = (position.x + position.y) % 3;
    out = uniforms.color_scheme.tiles[index];

    let flags = get_flags(position);
    if (flags & SELECTED) != 0 {
        let alpha = uniforms.color_scheme.selected.w;
        return uniforms.color_scheme.selected * alpha + out * (1.0 - alpha);
        // return uniforms.color_scheme.selected;
    }

    if (flags & HIGHLIGHTED) != 0 {
        let alpha = uniforms.color_scheme.highlighted.w;
        return uniforms.color_scheme.highlighted * alpha + out * (1.0 - alpha);
    }    

    return out;
}

const NUMBER_OF_TILES: u32 = 91;

/// Maximum allowed value of a coordinate.
const MAX: u32 = 10;

// Maximum allowed value for a rank
const MAX_RANK: u32 = 20;

/// Maximum allowed absolute difference between coordinates.
const WIDTH: u32 = 5;


// The distance between centers
const STEP_SIZE: vec2f = vec2f(
    sqrt(3.0),
    1.0
) * 2.0 / f32(MAX_RANK / 2 + 1); // Board takes up more space vertically, so we need to adjust based on the number of ranks.
const OFFSET: vec2f = STEP_SIZE / 2.0;


/// Returns the nearest center of a grid centered on `grid_center`
//
// Possible values are `grid_center + vec2f(x * step_size.x, y * step_size.y)` for any value `x` and `y`.
fn nearest_center(uv: vec2f, grid_center: vec2f, step_size: vec2f) -> vec2f {
    // Align to center
    var out = uv - grid_center;

    // We want to make it so that `(step_size.x, 0)` and `(0, step_size.y)` are at `(1, 0)` and `(0, 1)`.
    // Therefore, we divide by step_size.
    out /= step_size;
    out = round(out);
    out *= step_size;

    // Add back the grid center
    out += grid_center;
    return out;
}


fn hexagonal_position(cart_center: vec2f, step_size: vec2f) -> vec2u {
    // [x, y] denotes hex, (x, y) denotes cart
    // [0, 0] is at (0, 0)
    // [1, 0] is at (-step_size.x, step_size.y)
    // [0, 1] is at (step_size.x, step_size.y)
    // So [x, y] is at ((y - x) * step_size.x, (x + y) * step_size.y)
    // 
    // Then,
    // - cart_center.x = (y - x) * step_size.x    {1}
    // - cart_center.y = (x + y) * step_size.y    {2}
    // 
    // We can solve for x and y
    // => y - x = cart_center.x / step_size.x
    // => y + x = cart_center.y / step_size.y
    // => 2y = cart_center.x / step_size.x + cart_center.y / step_size.y
    // vvv
    let unit = vec2i(cart_center / step_size);
    let y = (unit.x + unit.y) / 2;
    let x = y - unit.x;
    return vec2u(vec2i(x, y) + vec2i(5, 5));
}

fn is_valid_position(position: vec2u) -> bool {
    // I feel there should be a better way to compute absolute difference of `u32`s.
    let abs_diff = u32(abs(i32(position.x - position.y)));
    return (abs_diff <= WIDTH) && (position.x <= MAX)  && (position.y <= MAX);
}

// Taken from `../../../core/src/coordinate.rs:187`
fn rank_width(rank: u32) -> u32 {
    return min(min(rank, MAX_RANK - rank) / 2, u32(2)) * 2 + 1 + rank % 2;
}

// Taken from `../../../core/src/board.rs:68`
fn index(position: vec2u) -> u32 {
    let rank = position.x + position.y;
    var tiles_before_rank: u32 = 0;
    for (var i: u32 = u32(0); i < rank; i += 1) {
        tiles_before_rank += rank_width(i);
    }


    // Taken from `../../../core/src/coordinate.rs:201`
    let h = i32(rank) - i32(MAX);
    let w = (i32(rank) - i32(WIDTH) + 1) / 2;
    let first_valid_y = u32(max(max(h, w), 0));

    let index_on_rank = position.y - first_valid_y;
    return tiles_before_rank + index_on_rank;
}

fn get_flags(position: vec2u) -> u32 {
    let actual_index = index(position);
    let array_index = actual_index / 4;
    let vec_index = actual_index % 4;
    // return uniforms.tile_flags[array_index][vec_index];
    return uniforms.tile_flags[array_index][vec_index];
}

/// Vertex shader, just makes a square.
@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    const SQUARE_POSITIONS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
    );

    var out: VertexOut;
    out.position = vec4<f32>(SQUARE_POSITIONS[v_idx], 0.0, 1.0);
    out.uv = SQUARE_POSITIONS[v_idx];

    return out;
}
