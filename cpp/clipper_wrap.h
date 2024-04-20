// Copyright (c) 2017 Laurent Zubiaur
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#include <stdint.h>

extern "C" {

typedef int64_t cInt;
typedef void Path, Paths, ClipperOffset, Clipper;

struct IntPoint { cInt X; cInt Y; };
struct IntRect { cInt left; cInt top; cInt right; cInt bottom; };

enum JoinType { jtSquare, jtRound, jtMiter };
enum EndType { etClosedPolygon, etClosedLine, etOpenButt, etOpenSquare, etOpenRound };

const char *cl_err_msg();

// Path

Path*     cl_path_new();
void      cl_path_free(Path *path);
IntPoint* cl_path_get(Path *path, int i);
bool      cl_path_add(Path *path, cInt x, cInt y);
int       cl_path_size(Path *path);
double    cl_path_area(const Path *path);
bool      cl_path_orientation(const Path *path);
void      cl_path_reverse(Path *path);
int       cl_path_point_in_polygon(const Path *path, cInt x, cInt y);
Paths*    cl_path_simplify(const Path *in,int fillType);
Path*     cl_path_clean_polygon(const Path *in, double distance);

// Paths

Paths* cl_paths_new();
void   cl_paths_free(Paths *paths);
Path*  cl_paths_get(Paths *paths, int i);
bool   cl_paths_add(Paths *paths, Path *path);
int    cl_paths_size(Paths *paths);

// ClipperOffset

ClipperOffset* cl_offset_new(double miterLimit, double roundPrecision);
void           cl_offset_free(ClipperOffset *co);
Paths*         cl_offset_path(ClipperOffset* co, Path *subj, double delta, JoinType joinType, EndType endType);
Paths*         cl_offset_paths(ClipperOffset* co, Paths *subj, double delta, JoinType joinType, EndType endType);
void           cl_offset_clear(ClipperOffset *co);

// Clipper

Clipper* cl_clipper_new(Clipper *cl);
void     cl_clipper_free(Clipper *cl);
void     cl_clipper_clear(Clipper *cl);
void     cl_clipper_reverse_solution(Clipper *cl, bool value);
void     cl_clipper_preserve_collinear(Clipper *cl, bool value);
void     cl_clipper_strictly_simple(Clipper *cl, bool value);
bool     cl_clipper_add_path(Clipper *cl,Path *path, int pt, bool closed, const char *err);
bool     cl_clipper_add_paths(Clipper *cl,Paths *paths, int pt, bool closed, const char *err);
Paths*   cl_clipper_execute(Clipper *cl,int clipType,int subjFillType,int clipFillType);
IntRect  cl_clipper_get_bounds(Clipper *cl);

}
