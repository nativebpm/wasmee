# Stage 1: Build the static assets
FROM node:20-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm install

COPY . .
RUN npm run build

# Stage 2: Serve the assets using Nginx
FROM nginx:alpine

# Copy built assets to Nginx html directory
COPY --from=builder /app/dist /usr/share/nginx/html

# Expose port 8080 (Cloud Run default port)
EXPOSE 8080

# Configure Nginx to listen on 8080 instead of 80
RUN sed -i 's/listen\(  *\)80;/listen 8080;/' /etc/nginx/conf.d/default.conf

CMD ["nginx", "-g", "daemon off;"]
