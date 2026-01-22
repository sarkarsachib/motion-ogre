#!/bin/bash
# Setup script for Motion Ogre backend development

set -e

echo "🔧 Setting up Motion Ogre backend..."

# Check Node.js version
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+"
    exit 1
fi

NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Node.js 18+ is required. Found version: $(node -v)"
    exit 1
fi

echo "✅ Node.js $(node -v) detected"

# Create data directory
mkdir -p backend/data
echo "✅ Created data directory"

# Copy environment file if it doesn't exist
if [ ! -f backend/.env ]; then
    cp backend/.env.example backend/.env
    echo "✅ Created .env file from .env.example"
    echo "⚠️  Please update backend/.env with your configuration"
else
    echo "✅ .env file already exists"
fi

# Install Node.js dependencies
echo "📦 Installing Node.js dependencies..."
cd backend
npm install

echo "✅ Node.js dependencies installed"

# Check Python availability
cd ..
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version | cut -d' ' -f2 | cut -d'.' -f1)
    if [ "$PYTHON_VERSION" -ge 3 ]; then
        echo "✅ Python $(python3 --version) detected"
        
        # Install Python agents
        echo "📦 Installing Python agents..."
        pip install -e . > /dev/null 2>&1 || echo "⚠️  Python package installation failed (may need virtual env)"
    else
        echo "⚠️  Python 3+ not found. Agent execution will not work."
    fi
else
    echo "⚠️  Python not found. Agent execution will not work."
fi

# Check Rust availability
if command -v cargo &> /dev/null; then
    echo "✅ Rust $(cargo --version | cut -d' ' -f2) detected"
    
    # Build Rust crates
    echo "🔨 Building Rust crates..."
    cargo build --release > /dev/null 2>&1 || echo "⚠️  Rust build failed (may need to fix compilation errors)"
else
    echo "⚠️  Rust not found. Rust FFI bridge will not work."
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "To start the backend:"
echo "  cd backend"
echo "  npm run dev"
echo ""
echo "For more information, see:"
echo "  - backend/README.md"
echo "  - BACKEND_README.md"
