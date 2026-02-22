# jammin + JamBrains SDK Example

This project was created with [jammin](https://github.com/FluffyLabs/jammin), FluffyLabs' toolbox for JAM service builders.

## What is jammin?

Learn more about jammin in the [official documentation](https://fluffylabs.dev/jammin/). This template uses the **JamBrains SDK** for building JAM services.

## Getting Started

First, install the jammin CLI tool by following the [installation guide](https://fluffylabs.dev/jammin/getting-started.html).

This project includes:
- Pre-configured JamBrains SDK service in `services/example`
- Build configuration via `jammin.build.yml`
- Ready-to-use development environment

## Available Commands

### Build Services

```bash
jammin build
```

Builds all services defined in your `jammin.build.yml` configuration.

### Run Tests

```bash
jammin test
```

Runs unit tests for your services.

## Project Structure

```
.
├── jammin.build.yml    # jammin configuration
└── services/
    └── example/        # Your JamBrains SDK service
```

## Learn More

- [jammin on github](https://github.com/FluffyLabs/jammin)
- [jammin on npm](https://www.npmjs.com/package/@fluffylabs/jammin)
- [jambrains sdk](https://github.com/JamBrains/service-sdk)

## Next Steps

1. Explore the example service in `services/example/`
2. Run `jammin build` to build your service
3. Customize the service to fit your needs
