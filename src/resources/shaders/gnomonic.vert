//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

#version 330 core

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

const vec3 GLOBE_TANGENT = vec3(1, 0, 0);
const vec3 PROJECTION_CENTER = vec3(0, 0, 0);

void main()
{
    float longitude = radians(lonlat_position.x);
    float latitude = radians(lonlat_position.y);

    vec3 original_position = vec3(
        cos(longitude) * cos(latitude),
        sin(longitude) * cos(latitude),
        sin(latitude)
    );

    vec3 position = globe_orientation * original_position;

    vec3 to_position = position - PROJECTION_CENTER;
    const vec3 to_tangent = GLOBE_TANGENT - PROJECTION_CENTER;

    if (to_position == vec3(0, 0, 0))
    {
        gl_Position = DISCARD;
        return;
    }

    float cos_side_angle = dot(to_position, to_tangent) / (length(to_position) * length(to_tangent));
    if (cos_side_angle < cos(radians(80)))
    {
        gl_Position = DISCARD;
        return;
    }

    float k = dot(to_tangent, to_tangent) / dot(to_position, to_tangent);
    vec3 projected = PROJECTION_CENTER + k * to_position;

    gl_Position = vec4(zoom / wh_ratio * projected.y, zoom * projected.z, 0, 1);
    vs_out.tex_coord = vec2(0.5 + lonlat_position.x / 360.0, 0.5 - lonlat_position.y / 180.0);
}
