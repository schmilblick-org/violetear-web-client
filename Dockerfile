FROM nginx
COPY target/deploy /usr/share/nginx/html
RUN sed -i "#application/javascript#a application/wasm  wasm;" /etc/nginx/mime.types