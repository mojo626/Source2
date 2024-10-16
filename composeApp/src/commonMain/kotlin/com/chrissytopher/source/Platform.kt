package com.chrissytopher.source

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.graphics.Shape
import org.jetbrains.compose.resources.DrawableResource

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform

expect fun getSourceData(username: String, password: String): Result<SourceData>

expect fun closeApp()

expect fun filesDir(): String

expect fun livingInFearOfBackGestures(): Boolean

expect fun appIcon(): DrawableResource
expect fun iconRounding(): RoundedCornerShape