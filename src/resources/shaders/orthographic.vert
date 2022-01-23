//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

#version 330 core

// looks from (1, 0, 0) at (0, 0, 0) with "up" being (0, 0, 1)
const mat4 VIEW = mat4(
    0, 0, 1, 0,
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, -1, 1
);

// corresponds to orthographic projection with x ∊ [-1, 1], y ∊ [-1, 1], z ∊ [0, 1]
const mat4 PROJECTION = mat4(
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, -2, 0,
    0, 0, -1, 1
);

uniform mat3 globe_orientation;
uniform float zoom;
uniform float wh_ratio;

in vec2 lonlat_position;
out VS_OUT
{
    vec2 tex_coord;
} vs_out;

// has to equal `DISCARD` in "*.geom"
const vec4 DISCARD = vec4(1.0e+9, 1.0e+9, 1.0e+9, 1.0e+9);

void main()
{
    float longitude = radians(lonlat_position.x);
    float latitude = radians(lonlat_position.y);

    vec3 position = vec3(
        cos(longitude) * cos(latitude),
        sin(longitude) * cos(latitude),
        sin(latitude)
    );

    mat4 view_model = VIEW * mat4(globe_orientation);
    vec4 view_model_position = view_model * vec4(position, 1.0);
    vec4 projected = PROJECTION * view_model_position;

    gl_Position.xy = vec2(projected.x * zoom / wh_ratio, projected.y * zoom);
    gl_Position.zw = projected.zw;
    vs_out.tex_coord = vec2(0.5 + lonlat_position.x / 360.0, 0.5 - lonlat_position.y / 180.0);
}
