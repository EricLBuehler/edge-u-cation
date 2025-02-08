package com.example.mistralrs.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

// Pastel Blue definitions for Light Theme
val PastelBluePrimary = Color(0xFFBBDEFB)           // Blue 100 – a gentle pastel blue
val PastelBlueOnPrimary = Color.Black
val PastelBluePrimaryContainer = Color(0xFFE3F2FD)    // Blue 50 – even lighter for containers

val PastelBlueSecondary = Color(0xFF90CAF9)          // Blue 200 – slightly more vibrant
val PastelBlueOnSecondary = Color.Black
val PastelBlueSecondaryContainer = Color(0xFFBBDEFB)   // Reusing Blue 100

val PastelBlueTertiary = Color(0xFF64B5F6)           // Blue 300 – a soft accent blue
val PastelBlueOnTertiary = Color.Black

private val LightColorScheme = lightColorScheme(
    primary = PastelBluePrimary,
    onPrimary = PastelBlueOnPrimary,
    primaryContainer = PastelBluePrimaryContainer,
    onPrimaryContainer = PastelBlueOnPrimary,
    secondary = PastelBlueSecondary,
    onSecondary = PastelBlueOnSecondary,
    secondaryContainer = PastelBlueSecondaryContainer,
    onSecondaryContainer = PastelBlueOnSecondary,
    tertiary = PastelBlueTertiary,
    onTertiary = PastelBlueOnTertiary,
    background = Color(0xFFFFFBFE),
    onBackground = Color(0xFF1C1B1F),
    surface = Color(0xFFFFFBFE),
    onSurface = Color(0xFF1C1B1F)
)

// Pastel Blue definitions for Dark Theme
val PastelBluePrimaryDark = Color(0xFF90CAF9)        // Blue 200 – light accent for dark mode
val PastelBlueOnPrimaryDark = Color(0xFF0D47A1)        // Dark blue for contrast
val PastelBluePrimaryContainerDark = Color(0xFF1E88E5) // A mid-tone blue for container use

val PastelBlueSecondaryDark = Color(0xFF64B5F6)       // Blue 300
val PastelBlueOnSecondaryDark = Color(0xFF0D47A1)
val PastelBlueSecondaryContainerDark = Color(0xFF1976D2)
val PastelBlueTertiaryDark = Color(0xFF42A5F5)
val PastelBlueOnTertiaryDark = Color.Black

private val DarkColorScheme = darkColorScheme(
    primary = PastelBluePrimaryDark,
    onPrimary = PastelBlueOnPrimaryDark,
    primaryContainer = PastelBluePrimaryContainerDark,
    onPrimaryContainer = Color.White,
    secondary = PastelBlueSecondaryDark,
    onSecondary = PastelBlueOnSecondaryDark,
    secondaryContainer = PastelBlueSecondaryContainerDark,
    onSecondaryContainer = Color.White,
    tertiary = PastelBlueTertiaryDark,
    onTertiary = PastelBlueOnTertiaryDark,
    background = Color(0xFF121212),
    onBackground = Color(0xFFE0E0E0),
    surface = Color(0xFF121212),
    onSurface = Color(0xFFE0E0E0)
)

@Composable
fun MistralrsTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    // Enable dynamic colors on Android 12+ if desired
    dynamicColor: Boolean = true,
    content: @Composable () -> Unit
) {
    val context = LocalContext.current
    val colorScheme = when {
//        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
//            if (darkTheme) dynamicDarkColorScheme(context)
//            else dynamicLightColorScheme(context)
//        }
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    // Update the status bar color for a cohesive look
    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            window.statusBarColor = colorScheme.primary.toArgb()
            WindowCompat.getInsetsController(window, view)?.isAppearanceLightStatusBars = !darkTheme
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography, // Use your custom typography
//        shapes = Shapes,         // Use your custom shapes
        content = content
    )
}
