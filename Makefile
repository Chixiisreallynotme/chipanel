# ==============================================================================
# ChiPanel Management Makefile
# ==============================================================================

.PHONY: all dev-backend dev-frontend build-frontend build-backend build-container deploy-quadlet clean

QUADLET_DIR ?= $(HOME)/.config/containers/systemd
CONTAINER_NAME ?= chipanel:latest

all: build-frontend build-backend

# Dev targets
dev-backend:
	cd backend && cargo run

dev-frontend:
	cd frontend && npm run dev

# Build targets
build-frontend:
	cd frontend && npm install && npm run build

build-backend:
	cd backend && cargo build --release

build-container:
	podman build -t $(CONTAINER_NAME) -f Containerfile .

# Deployment targets
deploy-quadlet:
	@if [ ! -f chipanel.container ]; then \
		echo "Error: chipanel.container not found."; \
		echo "Create it from example first: cp chipanel.container.example chipanel.container"; \
		echo "Then set your ADMIN_PASSWORD and RCON_PASSWORD in chipanel.container"; \
		exit 1; \
	fi
	mkdir -p $(QUADLET_DIR)
	cp chipanel.container $(QUADLET_DIR)/chipanel.container
	systemctl --user daemon-reload
	@echo "Quadlet deployed. Manage service via: systemctl --user status chipanel"

# Cleanup
clean:
	rm -rf backend/target
	rm -rf frontend/build
	rm -rf frontend/.svelte-kit
	rm -rf frontend/node_modules
