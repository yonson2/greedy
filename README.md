# Greedy

A high-performance<sup>*</sup> image processing service that handles resizing,
format conversion, and caching of images with an efficient in-memory storage
system.

Let's say we have an image `https://example.org/image.webp` and we are including it a website in a typical way:

```html
    <img src="https://greedy.org/image.webp" alt=""/>
```

This is probably fine, but what if we wanted to score some points on google
pagespeed and serve different images depending on the view-port while also
offering different image formats? One way is to manually convert your image
to the desired sizes and formats but that takes too much time and resources.

Instead, just deploy greedy somewhere (greedy.org in this example) and now
you magically have all of your desired images and more, your old `<img>` tag
can now be turned into this monster:

```html
<picture>
  <source media="(max-width:640px)" srcset="greedy.org/https://example.org/img.webp?&width=640" type="image/webp"> // keep original format
  <source media="(max-width:640px)" srcset="greedy.org/https://example.org/img.webp?format=avif&width=640" type="image/avif">
  <source media="(max-width:640px)" srcset="greedy.org/https://example.org/img.webp?format=png&width=640" type="image/png">
  <source media="(max-width:768px)" srcset="greedy.org/https://example.org/img.webp?format=webp&width=768" type="image/webp">
  <source media="(max-width:768px)" srcset="greedy.org/https://example.org/img.webp?format=avif&width=768" type="image/avif">
  <source media="(max-width:768px)" srcset="greedy.org/https://example.org/img.webp?format=png&width=768" type="image/png">
  <source media="(max-width:1024px)" srcset="greedy.org/https://example.org/img.webp?format=webp&width=1024" type="image/webp">
  <source media="(max-width:1024px)" srcset="greedy.org/https://example.org/img.webp?format=avif&width=1024" type="image/avif">
  <source media="(max-width:1024px)" srcset="greedy.org/https://example.org/img.webp?format=png&width=1024" type="image/png">
  <source media="(min-width:1280px)" srcset="https://example.org/img.webp" type="image/webp">
  <source media="(min-width:1280px)" srcset="greedy.org/https://example.org/img.webp?format=avif" type="image/avif"> // keep original size
  <source media="(min-width:1280px)" srcset="greedy.org/https://example.org/img.webp?format=png" type="image/png">
  <img src="https://example.org/img.webp">
</picture>
```

## Deployment

[greedy](https://github.com/yonson2/greedy) is a small rust app, and so can be
deployed to any provider that accepts rust apps by default; a dockerfile
is also included for convenience.

Take a look into `config/default.toml` and modify to your needs, its specially
important that you modify the `whitelist` parameter, otherwise the images
you try to send to it will get rejected.

Configuration parameters can be overwritten via environment variables or
environment-based configuration files, that is, if `RUST_ENV` is set to
`production` greedy will look for a configuration file
`config/production.toml` in the working directory of the app.

## Endpoints

### GET: `/:url`

Optional query parameters: `width`, `height` and `format`

This is the main endpoint of the app, fetches the image found on `:url` and
resizes/transforms the image if any combination of the query parameters
are provided, otherwise it will just cache the image as is.

### POST: /preload

Accepts a json body with the following payload:

```json
{
  "url": "https://the_url_of_your_image",
  "width": 500 //optional,
  "height": 500 //optional,
  "format": "png" //optional,
}
```

This acts as if a request to the get endpoint came in, preparing the image and
inserting it in the cache, making it so sub-sequent requests will load faster.
