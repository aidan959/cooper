#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_GOOGLE_include_directive : enable

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
   vec4 base_color_factor;
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
    // render settings
    uint shadows_enabled;
    uint ssao_enabled;
    uint fxaa_enabled;
    uint cubemap_enabled;
    uint ibl_enabled;
} view;

#define ENABLE_UV_Y_FLIP
#ifdef ENABLE_UV_Y_FLIP
    #define FLIP_UV_Y(uv) vec2(uv.x, 1.0 - uv.y)
#else
    #define FLIP_UV_Y(uv) uv
#endif

float luminance(vec3 rgb)
{
   // Coefficents from the BT.709 standard
   return dot(rgb, vec3(0.2126f, 0.7152f, 0.0722f));
}

float linearToSrgb(float linearColor)
{
   if (linearColor < 0.0031308f) {
      return linearColor * 12.92f;
   }
   else {
      return 1.055f * float(pow(linearColor, 1.0f / 2.4f)) - 0.055f;
   }
}

vec3 linearToSrgb(vec3 linearColor)
{
   return vec3(linearToSrgb(linearColor.x), linearToSrgb(linearColor.y), linearToSrgb(linearColor.z));
}

vec3 extract_camera_position(mat4 viewMatrix) {
   mat4 inverseViewMatrix = inverse(viewMatrix);
   vec3 cameraPosition = vec3(inverseViewMatrix[3]);
   return cameraPosition;
}

vec3 world_dir_from_ndc(vec3 ndc, mat4 view, mat4 projection)
{
   vec4 clipSpace = vec4(ndc, 1.0);
   vec4 viewSpace = inverse(projection) * clipSpace;
   viewSpace.w = 0.0;
   vec4 worldSpace = inverse(view) * viewSpace;
   vec3 worldDir = normalize(worldSpace.xyz);

   return worldDir;
}

vec3 world_dir_from_uv(vec2 uv, mat4 view, mat4 projection)
{
   return world_dir_from_ndc(vec3(uv, 0.0) * 2.0 - 1.0, view, projection);
}


layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec2 in_uv;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec4 in_color;
layout (location = 4) in vec4 in_tangent;
layout (location = 5) in mat3 in_tbn;

layout (location = 0) out vec4 out_gbuffer_position;
layout (location = 1) out vec4 out_gbuffer_albedo;
layout(push_constant) uniform PushConsts {
    mat4 world;
    vec4 color;
    uint mesh_index;
    ivec3 pad;
} pushConsts;

void main()
{
    Mesh mesh = meshesSSBO.meshes[pushConsts.mesh_index];
    Material material = materialsSSBO.materials[mesh.material];

    vec4 diffuse_color = texture(samplerColor[material.diffuse_map], in_uv);
    vec4 normal_map = texture(samplerColor[material.normal_map], in_uv);

    out_gbuffer_position = vec4(in_pos, 1.0);
    out_gbuffer_albedo = vec4(diffuse_color.rgb , 1.0);
}
