package com.chrissytopher.source

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

const val SOURCE_DATA_KEY = "SOURCE_DATA"
const val USERNAME_KEY = "USERNAME"
const val PASSWORD_KEY = "PASSWORD"
const val HIDE_MENTORSHIP_KEY = "HIDE_MENTORSHIP"
const val CLASS_UPDATES_KEY = "CLASS_UPDATES"
const val NEW_ASSIGNMENTS_NOTIFICATIONS_KEY = "NEW_ASSIGNMENTS_NOTIFICATIONS"
const val LETTER_GRADE_CHANGES_NOTIFICATIONS_KEY = "LETTER_GRADE_CHANGES_NOTIFICATIONS"
const val THRESHOLD_NOTIFICATIONS_KEY = "THRESHOLD_NOTIFICATIONS"
const val THRESHOLD_VALUE_NOTIFICATIONS_KEY = "THRESHOLD_VALUE_NOTIFICATIONS"

const val MENTORSHIP_NAME = "MENTORSHIP"

const val ASSIGNMENTS_NOTIFICATION_CHANNEL = "ASSIGNMENTS"

const val WORK_MANAGER_BACKGROUND_SYNC_ID = "BACKGROUND_SYNC"

const val NEW_ASSIGNMENTS_TITLE = "New Assignments"
const val NEW_ASSIGNMENTS_BODY = "Your assignments have been updated - Tap to view"

val gradeColors = mapOf(
    "A" to Color(0xFF64ed72),
    "B" to Color(0xFF69d0f5),
    "C" to Color(0xFFf0e269),
    "D" to Color(0xFFf09151),
    "E" to Color(0xFFf24646),
)

@Composable
fun darkModeColorModifier() = if (isSystemInDarkTheme()) 0.8f else 1f