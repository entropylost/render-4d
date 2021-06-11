#version 460

struct VoxelType {
    vec3 color;
};

struct Camera {
    vec3 position;
    mat3 inv_rotation;
    float tan_half_fov;
};

struct Ray {
    vec3 origin;
    vec3 direction;
};

layout (location = 0) out vec4 frag_color;

layout (set = 0, binding = 0) uniform Uniforms {
    Camera u_camera;
    vec2 window_size;
    VoxelType[256] types;
};
layout (set = 1, binding = 0) uniform utexture3D t_view;
layout (set = 1, binding = 1) uniform sampler s_view;

Ray generate_ray() {
    vec2 pixel_ndc = (gl_FragCoord.xy) / window_size;
    vec2 pixel_camera = 2 * pixel_ndc - 1;
    pixel_camera *= vec2(window_size.x / window_size.y, 1);
    pixel_camera *= u_camera.tan_half_fov;

    vec3 unnorm_dir = u_camera.inv_rotation * vec3(pixel_camera, 1);
    Ray ray;
    ray.origin = u_camera.position;
    ray.direction = normalize(unnorm_dir);
    return ray;
}

VoxelType get_voxel(ivec3 location) {
    return types[texelFetch(usampler3D(t_view, s_view), location, 0).x];
}

bool contains_voxel(ivec3 location) {
    return texelFetch(usampler3D(t_view, s_view), location, 0).x != 0;
}


// https://www.shadertoy.com/view/4dX3zl
void main() {
    Ray ray = generate_ray();

    // Position of the voxel the ray is currently in (integer)
    ivec3 grid_pos = ivec3(floor(ray.origin));

    // The amount you need to go along the ray to increment the voxel by one.
    vec3 delta_dist = abs(vec3(1) / ray.direction);

    // The direction of the ray as a sign.
    ivec3 ray_step = ivec3(sign(ray.direction));

    // sign(ray.direction) * 0.5 + 0.5: 0 if the direction is negative, 1 if its positive, per coord.
    // vec3(grid_pos) - ray.origin: Fractional part of the ray origin.
    // The distance to the next voxel along all 3 directions.
    vec3 side_dist = (sign(ray.direction) * (vec3(grid_pos) - ray.origin) + sign(ray.direction) * 0.5 + 0.5) * delta_dist;

    bvec3 mask;

    for (int i = 0; i < 128 * 3; i++) {
        if (contains_voxel(grid_pos)) break;

        mask = lessThanEqual(side_dist.xyz, min(side_dist.yzx, side_dist.zxy));

        side_dist += vec3(mask) * delta_dist;
        grid_pos += ivec3(vec3(mask)) * ray_step;
    }

    VoxelType voxel = get_voxel(grid_pos);

    float shadow;
    if (mask.x) {
        shadow = 0.5;
    }
    if (mask.y) {
        shadow = 1.0;
    }
    if (mask.z) {
        shadow = 0.75;
    }

    frag_color = vec4(contains_voxel(grid_pos) ? shadow * voxel.color : vec3(0), 1);
}
