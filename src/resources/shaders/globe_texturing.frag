//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

#version 330 core

in GS_OUT
{
    vec2 tex_coord;
} fs_in;
out vec4 output_color;

uniform sampler2D source_texture;

void main()
{
    output_color = texture(source_texture, fs_in.tex_coord);
}
