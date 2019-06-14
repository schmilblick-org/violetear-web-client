FROM nginx
COPY target/deploy /usr/share/nginx/html
RUN sed -i "s#}#    application\/wasm                                 wasm;\n}#" /etc/nginx/mime.types