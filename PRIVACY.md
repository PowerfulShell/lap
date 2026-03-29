# Privacy

Last updated: March 28, 2026

## Overview

Lap is designed as a local photo manager. Your photo library is processed on your device, and your files remain under your control.

This document explains what data Lap accesses, what data Lap does not collect by default, and when network access may occur.

## What Lap Accesses

Lap may access the following data on your device in order to provide its features:

- Photos, videos, and folders that you choose to add to a library
- File metadata such as filenames, paths, timestamps, size, format, EXIF data, ratings, tags, comments, and rotation state
- Generated local app data such as thumbnails, indexes, search data, embeddings, clustering data, and other library-related cache or database records

Lap uses this data to support browsing, search, deduplication, tagging, ratings, face clustering, and other library management features.

## Local Processing

Lap is intended to process your library locally on your device.

By default:

- Your photos and videos are not uploaded to a Lap cloud service
- Lap does not include advertising trackers
- Lap does not send telemetry or analytics about your library usage

## Network Access

Lap may access the network in limited cases where the feature requires it. Based on the current implementation, this may include:

- Checking for application updates
- Downloading application updates from GitHub releases when you choose to install an update
- Opening external links such as the project website or GitHub repository in your browser

If anonymous usage statistics is enabled, Lap may also send a very small set of anonymous product events through Aptabase, as described below.

If a future feature requires additional network access, it should be documented in the release notes and relevant product documentation.

## Anonymous Usage Statistics

Lap can send anonymous usage statistics to help improve Lap's stability and user experience. This feature is controlled by the `Share anonymous usage statistics` toggle in `Settings > Privacy`.

Lap uses Aptabase, a privacy-first analytics service for apps. Events are anonymous and do not include your photos, file paths, or personal identifiers.

When enabled, Lap sends only:

- Anonymous app activity counters
- Lap version
- Device platform and operating system
- A coarse region inferred by the analytics service from network metadata

## Data Storage

Lap stores application data locally on your device. This may include:

- App settings
- Local database including generated thumbnails and related indexing data

This local data is used to provide the app's functionality and improve performance on your device.

## Third-Party Services

Lap does not provide its own cloud storage service.

When you use update-related features, release assets may be fetched from GitHub. Those requests are subject to GitHub's terms and privacy practices.

When anonymous analytics is enabled, event delivery is handled by Aptabase. Those requests are subject to Aptabase's terms and privacy practices.

## Changes to This Document

This document may be updated as the app evolves. Privacy-related changes should be reflected in the repository and user-facing documentation.

## Contact

For privacy-related questions or concerns, please open an issue in the project repository:

[https://github.com/julyx10/lap](https://github.com/julyx10/lap)
