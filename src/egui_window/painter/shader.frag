
uniform sampler2D u_sampler;

#if __VERSION__ < 130
#define VARYING_IN varying
#define FRAG_COLOR gl_FragColor
#define SAMPLE texture2D
#else
precision highp float;
out vec4 f_color;
#define VARYING_IN in
#define FRAG_COLOR f_color
#define SAMPLE texture
#endif

VARYING_IN vec4 v_rgba_in_gamma;
VARYING_IN vec2 v_tc;

void main() {
  vec4 texture_in_gamma = SAMPLE(u_sampler, v_tc);

  // We multiply the colors in gamma space, because that's the only way to get text to look right.
  vec4 frag_color_gamma = v_rgba_in_gamma * texture_in_gamma;

  FRAG_COLOR = frag_color_gamma;
}
