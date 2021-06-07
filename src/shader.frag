#version 460

struct VoxelType {
    vec4 color;
};

struct Camera {
    vec3 position;
    mat3 rotation;
    vec2 screen_size;
    float aspect_ratio;
    float tan_half_fov;
};

struct Ray {
    vec3 origin;
    vec3 direction;
};

layout (location = 0) out vec4 frag_color;

layout (set = 0, binding = 0) uniform Uniforms {
    Camera u_camera;
    VoxelType[256] types;
};
layout (set = 1, binding = 0) uniform utexture3D t_world_3d;
layout (set = 1, binding = 1) uniform sampler s_world_3d;

Ray generate_ray() {
    // Also adding 0.5 here, because we want it to stay in the middle of the pixel
    vec2 pixel_ndc = (gl_FragCoord.xy) / u_camera.screen_size;
    vec2 pixel_camera = 2 * pixel_ndc - 1;
    pixel_camera.x *= u_camera.aspect_ratio;
    pixel_camera *= u_camera.tan_half_fov;

    vec3 unnorm_dir = u_camera.rotation * vec3(pixel_camera, -1);
    Ray ray;
    ray.origin = u_camera.position;
    ray.direction = normalize(unnorm_dir);
    return ray;
}

VoxelType get_voxel(ivec3 location) {
    return types[texelFetch(usampler3D(t_world_3d, s_world_3d), location, 0).x];
}

bool contains_voxel(ivec3 location) {
    return texelFetch(usampler3D(t_world_3d, s_world_3d), location, 0).x != 0;
}

void main() {

    Ray ray = generate_ray();

    ivec3 map_pos = ivec3(floor(ray.origin + 0.));

    vec3 delta_dist = abs(vec3(length(ray.direction)) / ray.direction);

    ivec3 ray_step = ivec3(sign(ray.direction));

    vec3 side_dist = (sign(ray.direction) * (vec3(map_pos) - ray.origin) + (sign(ray.direction) * 0.5) + 0.5) * delta_dist;

    bvec3 mask;

    for (int i = 0; i < 8; i++) {
        if (contains_voxel(map_pos)) break;
        //Thanks kzy for the suggestion!
        mask = lessThanEqual(side_dist.xyz, min(side_dist.yzx, side_dist.zxy));
        /*bvec3 b1 = lessThan(sideDist.xyz, sideDist.yzx);
        bvec3 b2 = lessThanEqual(sideDist.xyz, sideDist.zxy);
        mask.x = b1.x && b2.x;
        mask.y = b1.y && b2.y;
        mask.z = b1.z && b2.z;*/
        //Would've done mask = b1 && b2 but the compiler is making me do it component wise.
        //All components of mask are false except for the corresponding largest component
        //of sideDist, which is the axis along which the ray should be incremented.

        side_dist += vec3(mask) * delta_dist;
        map_pos += ivec3(vec3(mask)) * ray_step;
    }

    VoxelType voxel = get_voxel(map_pos);

    frag_color = voxel.color;
    if (! contains_voxel(map_pos)) {
        frag_color = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
