
#if __VERSION__ < 130
#define IN attribute
#define OUT varying
#else
precision highp float;
#define IN in
#define OUT out
#endif

IN vec2 a_pos;
IN vec4 a_srgba; // 0-255 sRGB
IN vec2 a_tc;

OUT vec4 v_rgba_in_gamma;
OUT vec2 v_tc;

uniform vec2 u_screen_size;

void main() {
  gl_Position = vec4(
      2.0 * a_pos.x / u_screen_size.x - 1.0,
      1.0 - 2.0 * a_pos.y / u_screen_size.y,
      0.0,
      1.0);
  v_rgba_in_gamma = a_srgba / 255.0;
  v_tc = a_tc;
}
