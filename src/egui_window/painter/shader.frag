#version 330

precision highp float;

uniform sampler2D u_sampler;

in vec4 v_rgba_in_gamma;
in vec2 v_tc;
out vec4 f_color;

void main() {
  vec4 texture_in_gamma = texture2D(u_sampler, v_tc);

  // We multiply the colors in gamma space, because that's the only way to get text to look right.
  vec4 frag_color_gamma = v_rgba_in_gamma * texture_in_gamma;

  gl_FragColor = frag_color_gamma;
}
