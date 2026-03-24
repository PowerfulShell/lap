#include "libraw/libraw.h"

#include <cstdlib>
#include <cstring>

extern "C" {

struct LapLibRawImage {
  unsigned char *data;
  unsigned int len;
  int format;
  unsigned short width;
  unsigned short height;
  unsigned short colors;
  unsigned short bits;
  int flip;
};

libraw_data_t *lap_libraw_open_buffer(const unsigned char *data, size_t len,
                                      int *err) {
  libraw_data_t *raw = libraw_init(0);
  if (!raw) {
    if (err) {
      *err = -1;
    }
    return nullptr;
  }

  // Use camera white balance and matrix for processed preview rendering.
  // These parameters are consulted during file open for matrix-backed cameras
  // such as many Nikon NEF samples, so they must be set before open_buffer().
  raw->params.use_camera_wb = 1;
  raw->params.use_camera_matrix = 1;

  int ret = libraw_open_buffer(raw, data, len);
  if (ret != LIBRAW_SUCCESS) {
    if (err) {
      *err = ret;
    }
    libraw_close(raw);
    return nullptr;
  }

  if (err) {
    *err = LIBRAW_SUCCESS;
  }
  return raw;
}

void lap_libraw_close(libraw_data_t *raw) {
  if (raw) {
    libraw_close(raw);
  }
}

const char *lap_libraw_strerror(int code) { return libraw_strerror(code); }

int lap_libraw_get_dimensions(libraw_data_t *raw, unsigned int *width,
                              unsigned int *height, int *flip) {
  if (!raw) {
    return LIBRAW_UNSPECIFIED_ERROR;
  }

  int ret = libraw_adjust_sizes_info_only(raw);
  if (ret != LIBRAW_SUCCESS) {
    return ret;
  }

  if (width) {
    *width = static_cast<unsigned int>(raw->sizes.iwidth ? raw->sizes.iwidth
                                                         : raw->sizes.width);
  }
  if (height) {
    *height = static_cast<unsigned int>(raw->sizes.iheight ? raw->sizes.iheight
                                                           : raw->sizes.height);
  }
  if (flip) {
    *flip = raw->sizes.flip;
  }

  return LIBRAW_SUCCESS;
}

int lap_libraw_get_thumbnail_count(libraw_data_t *raw) {
  if (!raw) {
    return 0;
  }
  return raw->thumbs_list.thumbcount;
}

int lap_libraw_extract_thumbnail(libraw_data_t *raw, int index,
                                 LapLibRawImage *out) {
  if (!raw || !out) {
    return LIBRAW_UNSPECIFIED_ERROR;
  }

  std::memset(out, 0, sizeof(*out));

  int ret = index >= 0 ? libraw_unpack_thumb_ex(raw, index) : libraw_unpack_thumb(raw);
  if (ret != LIBRAW_SUCCESS) {
    return ret;
  }

  const auto &thumb = raw->thumbnail;
  out->format = static_cast<int>(thumb.tformat);
  out->width = thumb.twidth;
  out->height = thumb.theight;
  out->colors = static_cast<unsigned short>(thumb.tcolors);
  out->bits = thumb.tformat == LIBRAW_THUMBNAIL_BITMAP16 ? 16 : 8;
  out->len = thumb.tlength;
  out->flip = raw->sizes.flip;

  if (index >= 0 && index < raw->thumbs_list.thumbcount) {
    int tflip = raw->thumbs_list.thumblist[index].tflip;
    if (tflip != 0xffff) {
      out->flip = tflip;
    }
  }

  if (thumb.tlength > 0 && thumb.thumb) {
    out->data = static_cast<unsigned char *>(std::malloc(thumb.tlength));
    if (!out->data) {
      return LIBRAW_UNSPECIFIED_ERROR;
    }
    std::memcpy(out->data, thumb.thumb, thumb.tlength);
  }

  return LIBRAW_SUCCESS;
}

int lap_libraw_render_preview(libraw_data_t *raw, int half_size,
                              LapLibRawImage *out) {
  if (!raw || !out) {
    return LIBRAW_UNSPECIFIED_ERROR;
  }

  std::memset(out, 0, sizeof(*out));

  if (half_size) {
    raw->params.half_size = 1;
  }

  int ret = libraw_unpack(raw);
  if (ret != LIBRAW_SUCCESS) {
    return ret;
  }

  libraw_set_output_bps(raw, 8);

  ret = libraw_dcraw_process(raw);
  if (ret != LIBRAW_SUCCESS) {
    return ret;
  }

  int err = LIBRAW_SUCCESS;
  libraw_processed_image_t *image = libraw_dcraw_make_mem_image(raw, &err);
  if (!image) {
    return err != LIBRAW_SUCCESS ? err : LIBRAW_UNSPECIFIED_ERROR;
  }

  out->format = static_cast<int>(image->type);
  out->width = image->width;
  out->height = image->height;
  out->colors = image->colors;
  out->bits = image->bits;
  out->len = image->data_size;
  out->flip = raw->sizes.flip;

  if (image->data_size > 0) {
    out->data = static_cast<unsigned char *>(std::malloc(image->data_size));
    if (!out->data) {
      libraw_dcraw_clear_mem(image);
      return LIBRAW_UNSPECIFIED_ERROR;
    }
    std::memcpy(out->data, image->data, image->data_size);
  }

  libraw_dcraw_clear_mem(image);
  return LIBRAW_SUCCESS;
}

void lap_libraw_free_buffer(unsigned char *data) {
  if (data) {
    std::free(data);
  }
}

}
