from setuptools import setup, find_packages

setup(
    name="cli-anything-arcanecodex",
    version="0.1.0",
    description="ArcaneCodex CLI for AI Agents - Local Image Knowledge Management",
    author="CLI-Anything Community",
    author_email="community@cli-anything.com",
    url="https://github.com/HKUDS/CLI-Anything",
    py_modules=["arcanecodex"],
    entry_points={
        "console_scripts": [
            "ac=arcanecodex:cli",
        ],
    },
    install_requires=[
        "click>=8.0",
    ],
    python_requires=">=3.8",
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
)
