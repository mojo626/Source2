package com.chrissytopher.source

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Percent
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import kotlinx.datetime.LocalDate
import kotlinx.datetime.LocalDateTime
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material.icons.outlined.ChevronLeft
import androidx.compose.foundation.layout.offset
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.foundation.clickable
import androidx.compose.ui.text.style.TextAlign
import kotlinx.serialization.encodeToString
import net.sergeych.sprintf.*

@Composable
fun GradesScreen() {
    val currentClass by ClassForGradePage.current
    var assignmentForPage = AssignmentForPage.current
    val kvault = LocalKVault.current
    val json = LocalJson.current

    LaunchedEffect(currentClass) {
        val updateClasses = kvault?.string(CLASS_UPDATES_KEY)?.let { json.decodeFromString<ArrayList<String>>(it) } ?: arrayListOf()
        currentClass?.name?.let { updateClasses.remove(it) }
        kvault?.set(CLASS_UPDATES_KEY, json.encodeToString(updateClasses))
    }

    val navHost = LocalNavHost.current

    var goToAssignment by remember { mutableStateOf<AssignmentSection?>(null) }

    LaunchedEffect(goToAssignment)
    {
        if (goToAssignment != null)
        {
            assignmentForPage.value = goToAssignment
            navHost?.navigate(NavScreen.Assignments.name)
            goToAssignment = null
        }
    }

    if (currentClass == null) {
        val navHost = LocalNavHost.current
        navHost?.popBackStack()
        return
    }
    val meta = key(currentClass) {
        remember { ClassMeta(currentClass!!) }
    }

    var goBack by remember { mutableStateOf(false) }
    LaunchedEffect (goBack) {
        if (goBack) {
            navHost?.navigate(NavScreen.Home.name)
        }
    }

    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
        Row( verticalAlignment = Alignment.CenterVertically ) {
            IconButton({ goBack = true }) {
                Icon(Icons.Outlined.ChevronLeft, contentDescription = "left arrow", modifier = Modifier.padding(5.dp))
            }
            
            Text(currentClass!!.name, style = MaterialTheme.typography.titleLarge, modifier = Modifier.weight(1f).offset(x= -12.dp).padding(5.dp), textAlign = TextAlign.Center)
        }
        
        Row {
            Box (modifier = Modifier.aspectRatio(1f).weight(1f).padding(10.dp)) {
                ClassCard(currentClass!!, meta, false)
            }
            Box (modifier = Modifier.aspectRatio(1f).weight(1f).padding(10.dp)) {
                Text("Does anyone remember what was here on the og source app?")
            }
        }
        var showPercent by remember { mutableStateOf(true) }
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(5.dp)) {
            Text("Assignments", style = MaterialTheme.typography.titleLarge, modifier = Modifier.weight(1f))
            Switch(
                checked = showPercent,
                onCheckedChange = { showPercent = it },
                thumbContent = {
                        Icon(
                            imageVector = Icons.Filled.Percent,
                            contentDescription = null,
                            modifier = Modifier.size(SwitchDefaults.IconSize),
                        )
                    }
            )

        }
        val assignmentsSorted = key(currentClass) {
            remember { currentClass?.assignments_parsed?.sortedByDescending { it._assignmentsections.maxOf { LocalDate.parse(it.duedate) } } }
        }
        Column(Modifier.fillMaxSize()) {
            assignmentsSorted?.forEach {assignment ->
                val newestSection =
                    assignment._assignmentsections.maxByOrNull { LocalDate.parse(it.duedate) }
                val newestScore = newestSection?._assignmentscores?.maxByOrNull { LocalDateTime.parse(it.scoreentrydate) }
                if (newestSection != null) {
                    val themeModifier = darkModeColorModifier()
                    val colors = if (newestSection.iscountedinfinalgrade) {
                        newestScore?.scorelettergrade?.firstOrNull()?.let {
                            gradeColors[it.toString()]?.let {
                                CardDefaults.cardColors(
                                    containerColor = it*themeModifier
                                )
                            }
                        } ?: CardDefaults.cardColors()
                    } else {
                        CardDefaults.cardColors()
                    }
                    Card(modifier = Modifier.padding(5.dp).clickable { goToAssignment = newestSection }, colors = colors) {
                        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(10.dp)) {
                            val localDensity = LocalDensity.current
                            Text(newestScore?.scorelettergrade ?: "", modifier = Modifier.width( with (localDensity) { MaterialTheme.typography.titleLarge.fontSize.toDp()*1.5f } ), style = MaterialTheme.typography.titleLarge)
                            Text(newestSection.name, style = MaterialTheme.typography.titleLarge, modifier = Modifier.weight(1f))
                            Text(
                                if (showPercent) {
                                    newestScore?.scorepercent?.let {"${"%.2f".sprintf(it)}%"} ?: "-"
                                } else {
                                    newestScore?.scorepoints?.let { "${it * newestSection.weight} / ${newestSection.totalpointvalue}" } ?: "-"
                                },
                                style = MaterialTheme.typography.titleLarge)
                        }
                    }
                }
            }
        }
    }
}