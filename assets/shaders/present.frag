#extension GL_EXT_scalar_block_layout : enable
#extension GL_EXT_nonuniform_qualifier : enable

struct Vertex
{
   vec4 pos;
   vec4 normal;
   vec2 uv;
   vec4 color;
   vec4 tangent;
};

struct Material
{
   uint diffuse_map;
   uint normal_map;
   uint metallic_roughness_map;
   uint occlusion_map;
   vec4 base_color_factor;
   float metallic_factor;
   float roughness_factor;
   vec2 padding;
};

struct Mesh
{
   uint vertex_buffer;
   uint index_buffer;
   uint material;
};  

layout (set = 0, binding = 0) uniform sampler2D samplerColor[];

layout (std430, set = 0, binding = 1) readonly buffer VerticesSSBO
{
   Vertex vertices[];
} verticesSSBO[];

layout (scalar, set = 0, binding = 2) readonly buffer IndicesSSBO
{
   ivec3 indices[];
} indicesSSBO[];

layout (scalar, set = 0, binding = 3) readonly buffer MaterialsSSBO
{
   Material materials[];
} materialsSSBO;

layout (scalar, set = 0, binding = 4) readonly buffer MeshesSSBO
{
   Mesh meshes[];
} meshesSSBO;

layout (std140, set = 1, binding = 0) uniform UBO_view
{
    mat4 view;
    mat4 projection;
    mat4 inverse_view;
    mat4 inverse_projection;
    vec3 eye_pos;
    vec3 sun_dir;
    uint viewport_width;
    uint viewport_height;
    uint shadows_enabled;
    uint ssao_enabled;
    uint fxaa_enabled;
    uint cubemap_enabled;
    uint ibl_enabled;
} view;

layout (location = 0) in vec2 in_uv;

layout (location = 0) out vec4 out_color;

layout (set = 2, binding = 0) uniform sampler2D in_color_texture;


void main() {
    vec2 uv = FLIP_UV_Y(in_uv);

    vec3 color = vec3(0.0);

    color = ;

    color = linearToSrgb(texture(in_color_texture, uv).rgb);

    out_color = vec4(color, 1.0);
}

