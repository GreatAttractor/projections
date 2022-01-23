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

const float PI = 3.14159;

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

    float r = sqrt(position.x * position.x + position.y * position.y);
    float angle = 0;
    if (r > 0)
    {
        angle = (position.x > 0) ?
            asin(position.y / r) :
            sign(position.y) * PI - asin(position.y / r);
    }

    gl_Position = vec4(zoom / wh_ratio * angle, zoom * position.z, 0, 1);

    vs_out.tex_coord = vec2(0.5 + lonlat_position.x / 360.0, 0.5 - lonlat_position.y / 180.0);
}
